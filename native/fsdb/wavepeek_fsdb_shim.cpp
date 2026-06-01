#include "wavepeek_fsdb_shim.h"

#include "ffrAPI.h"

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <algorithm>
#include <exception>
#include <memory>
#include <mutex>
#include <new>
#include <string>
#include <unordered_map>
#include <utility>
#include <unordered_set>
#include <vector>

#include <fcntl.h>
#include <unistd.h>

struct wp_fsdb_signal_session;

struct wp_fsdb_reader {
    ffrObject *object;
    wp_fsdb_signal_session *active_signal_session;
};

namespace {

bool native_verbose_enabled() {
    const char *value = std::getenv("WAVEPEEK_FSDB_NATIVE_VERBOSE");
    return value != nullptr && std::strcmp(value, "1") == 0;
}

std::recursive_mutex &reader_mutex() {
    static std::recursive_mutex mutex;
    return mutex;
}

class scoped_output_suppressor {
  public:
    scoped_output_suppressor() {
        if (native_verbose_enabled()) {
            return;
        }

        std::fflush(stdout);
        std::fflush(stderr);

        devnull_ = open("/dev/null", O_WRONLY);
        if (devnull_ < 0) {
            return;
        }

        saved_stdout_ = dup(STDOUT_FILENO);
        saved_stderr_ = dup(STDERR_FILENO);
        if (saved_stdout_ >= 0) {
            dup2(devnull_, STDOUT_FILENO);
        }
        if (saved_stderr_ >= 0) {
            dup2(devnull_, STDERR_FILENO);
        }
    }

    scoped_output_suppressor(const scoped_output_suppressor &) = delete;
    scoped_output_suppressor &operator=(const scoped_output_suppressor &) = delete;

    ~scoped_output_suppressor() {
        if (devnull_ < 0) {
            return;
        }

        std::fflush(stdout);
        std::fflush(stderr);

        if (saved_stdout_ >= 0) {
            dup2(saved_stdout_, STDOUT_FILENO);
            close(saved_stdout_);
        }
        if (saved_stderr_ >= 0) {
            dup2(saved_stderr_, STDERR_FILENO);
            close(saved_stderr_);
        }
        close(devnull_);
    }

  private:
    int devnull_ = -1;
    int saved_stdout_ = -1;
    int saved_stderr_ = -1;
};

void suppress_reader_messages() {
    ffrObject::ffrInfoSuppress(1);
    ffrObject::ffrWarnSuppress(1);
}

char *copy_string(const char *value) {
    if (value == nullptr) {
        value = "";
    }

    const std::size_t length = std::strlen(value);
    char *copy = static_cast<char *>(std::malloc(length + 1));
    if (copy == nullptr) {
        return nullptr;
    }

    std::memcpy(copy, value, length + 1);
    return copy;
}

wp_fsdb_status fail(char **error_message, const std::string &message) {
    if (error_message != nullptr) {
        *error_message = copy_string(message.c_str());
    }
    return WP_FSDB_STATUS_ERROR;
}

wp_fsdb_status fail_unknown_exception(char **error_message) {
    return fail(error_message, "FSDB Reader: native shim caught an unknown C++ exception");
}

uint64_t tag64_to_u64(const fsdbTag64 &tag) {
    return (static_cast<uint64_t>(tag.H) << 32) | static_cast<uint64_t>(tag.L);
}

fsdbTag64 u64_to_tag64(uint64_t value) {
    fsdbTag64 tag{};
    tag.H = static_cast<uint32_t>(value >> 32);
    tag.L = static_cast<uint32_t>(value & 0xffffffffu);
    return tag;
}

wp_fsdb_status require_integer_time_tags(ffrObject *object, char **error_message) {
    if (object == nullptr) {
        return fail(error_message, "FSDB Reader: time tag validation received a null reader");
    }

    const fsdbXTagType xtag_type = object->ffrGetXTagType();
    if (xtag_type != FSDB_XTAG_TYPE_L && xtag_type != FSDB_XTAG_TYPE_HL) {
        return fail(
            error_message,
            "FSDB Reader: unsupported non-integer FSDB time tag representation"
        );
    }
    return WP_FSDB_STATUS_OK;
}

struct free_deleter {
    void operator()(char *value) const {
        std::free(value);
    }
};

using owned_c_string = std::unique_ptr<char, free_deleter>;

void clear_error(char **error_message) {
    if (error_message != nullptr) {
        *error_message = nullptr;
    }
}

wp_fsdb_status reject_active_signal_session(
    wp_fsdb_reader *reader,
    const char *operation,
    char **error_message
) {
    if (reader != nullptr && reader->active_signal_session != nullptr) {
        return fail(
            error_message,
            std::string("FSDB Reader: cannot perform one-shot ") + operation +
                " while an FSDB signal session is active"
        );
    }
    return WP_FSDB_STATUS_OK;
}

wp_fsdb_scope_kind map_scope_kind(uint_T raw_type) {
    switch (raw_type) {
    case FSDB_ST_VCD_MODULE:
    case FSDB_ST_SV_MODULE:
    case FSDB_ST_SC_MODULE:
        return WP_FSDB_SCOPE_KIND_MODULE;
    case FSDB_ST_VCD_TASK:
        return WP_FSDB_SCOPE_KIND_TASK;
    case FSDB_ST_VCD_FUNCTION:
    case FSDB_ST_VHDL_FUNCTION:
        return WP_FSDB_SCOPE_KIND_FUNCTION;
    case FSDB_ST_VCD_BEGIN:
        return WP_FSDB_SCOPE_KIND_BEGIN;
    case FSDB_ST_VCD_FORK:
        return WP_FSDB_SCOPE_KIND_FORK;
    case FSDB_ST_VCD_GENERATE:
    case FSDB_ST_VHDL_GENERATE:
    case FSDB_ST_VHDL_FOR_GENERATE:
    case FSDB_ST_VHDL_IF_GENERATE:
        return WP_FSDB_SCOPE_KIND_GENERATE;
    case FSDB_ST_VHDL_RECORD:
        return WP_FSDB_SCOPE_KIND_STRUCT;
    case FSDB_ST_SV_CLASS:
        return WP_FSDB_SCOPE_KIND_CLASS;
    case FSDB_ST_SV_INTERFACE:
    case FSDB_ST_SV_MODPORT:
    case FSDB_ST_SV_INTERFACEPORT_REF:
    case FSDB_ST_SV_MODPORT_REF:
    case FSDB_ST_SV_INTERFACE_PORT:
    case FSDB_ST_SV_MODPORT_PORT:
        return WP_FSDB_SCOPE_KIND_INTERFACE;
    case FSDB_ST_SV_PACKAGE:
        return WP_FSDB_SCOPE_KIND_PACKAGE;
    case FSDB_ST_SV_PROGRAM:
        return WP_FSDB_SCOPE_KIND_PROGRAM;
    default:
        return WP_FSDB_SCOPE_KIND_UNKNOWN;
    }
}

wp_fsdb_signal_kind map_signal_kind(uint_T raw_type) {
    const uint_T type = raw_type & ~static_cast<uint_T>(FSDB_VT_MC_MASK);
    switch (type) {
    case FSDB_VT_VCD_EVENT:
    case FSDB_VT_EVENT_VARIABLE:
        return WP_FSDB_SIGNAL_KIND_EVENT;
    case FSDB_VT_VCD_INTEGER:
        return WP_FSDB_SIGNAL_KIND_INTEGER;
    case FSDB_VT_VCD_PARAMETER:
        return WP_FSDB_SIGNAL_KIND_PARAMETER;
    case FSDB_VT_VCD_REAL:
        return WP_FSDB_SIGNAL_KIND_REAL;
    case FSDB_VT_VCD_REG:
    case FSDB_VT_VCD_REG2:
        return WP_FSDB_SIGNAL_KIND_REG;
    case FSDB_VT_VCD_SUPPLY0:
        return WP_FSDB_SIGNAL_KIND_SUPPLY0;
    case FSDB_VT_VCD_SUPPLY1:
        return WP_FSDB_SIGNAL_KIND_SUPPLY1;
    case FSDB_VT_VCD_TIME:
        return WP_FSDB_SIGNAL_KIND_TIME;
    case FSDB_VT_VCD_TRI:
        return WP_FSDB_SIGNAL_KIND_TRI;
    case FSDB_VT_VCD_TRIAND:
        return WP_FSDB_SIGNAL_KIND_TRIAND;
    case FSDB_VT_VCD_TRIOR:
        return WP_FSDB_SIGNAL_KIND_TRIOR;
    case FSDB_VT_VCD_TRIREG:
        return WP_FSDB_SIGNAL_KIND_TRIREG;
    case FSDB_VT_VCD_TRI0:
        return WP_FSDB_SIGNAL_KIND_TRI0;
    case FSDB_VT_VCD_TRI1:
        return WP_FSDB_SIGNAL_KIND_TRI1;
    case FSDB_VT_VCD_WAND:
        return WP_FSDB_SIGNAL_KIND_WAND;
    case FSDB_VT_VCD_WIRE:
        return WP_FSDB_SIGNAL_KIND_WIRE;
    case FSDB_VT_VCD_WOR:
        return WP_FSDB_SIGNAL_KIND_WOR;
    case FSDB_VT_STRING:
        return WP_FSDB_SIGNAL_KIND_STRING;
    case FSDB_VT_VCD_PORT:
        return WP_FSDB_SIGNAL_KIND_PORT;
    case FSDB_VT_VCD_MEMORY:
    case FSDB_VT_VHDL_MEMORY:
        return WP_FSDB_SIGNAL_KIND_SPARSE_ARRAY;
    case FSDB_VT_AMS_SIGNAL:
        return WP_FSDB_SIGNAL_KIND_REAL;
    case FSDB_VT_VHDL_SIGNAL:
    case FSDB_VT_VHDL_VARIABLE:
    case FSDB_VT_VHDL_CONSTANT:
    case FSDB_VT_SV_VARIABLE:
        return WP_FSDB_SIGNAL_KIND_LOGIC;
    default:
        return WP_FSDB_SIGNAL_KIND_UNKNOWN;
    }
}

bool is_known_non_vector_signal(wp_fsdb_signal_kind kind) {
    switch (kind) {
    case WP_FSDB_SIGNAL_KIND_EVENT:
    case WP_FSDB_SIGNAL_KIND_REAL:
    case WP_FSDB_SIGNAL_KIND_STRING:
    case WP_FSDB_SIGNAL_KIND_SPARSE_ARRAY:
    case WP_FSDB_SIGNAL_KIND_REAL_TIME:
    case WP_FSDB_SIGNAL_KIND_REAL_PARAMETER:
    case WP_FSDB_SIGNAL_KIND_SHORT_REAL:
        return true;
    default:
        return false;
    }
}

wp_fsdb_value_encoding classify_value_encoding(
    const fsdbTreeCBDataVar *var,
    wp_fsdb_signal_kind kind
) {
    if (var == nullptr || is_known_non_vector_signal(kind)) {
        return WP_FSDB_VALUE_ENCODING_UNSUPPORTED;
    }
    if (var->is_dummy_var || var->is_class_in_obj || var->is_void || var->is_unexpanded_mem_var) {
        return WP_FSDB_VALUE_ENCODING_UNSUPPORTED;
    }
    if (kind == WP_FSDB_SIGNAL_KIND_UNKNOWN) {
        return WP_FSDB_VALUE_ENCODING_DATATYPE_CANDIDATE;
    }
    return WP_FSDB_VALUE_ENCODING_BIT_VECTOR;
}

wp_fsdb_datatype_kind map_datatype_kind(fsdbTreeCBType type) {
    switch (type) {
    case FSDB_TREE_CBT_DT_ENUM:
    case FSDB_TREE_CBT_DT_ENUM2:
    case FSDB_TREE_CBT_DT_ENUM3:
    case FSDB_TREE_CBT_DT_ATTR_ENUM:
    case FSDB_TREE_CBT_DT_ATTR_SV_ENUM:
        return WP_FSDB_DATATYPE_KIND_ENUM;
    case FSDB_TREE_CBT_DT_INT:
    case FSDB_TREE_CBT_DT_INT_H_N_L:
        return WP_FSDB_DATATYPE_KIND_INT;
    case FSDB_TREE_CBT_DT_ATTR_LOGIC:
    case FSDB_TREE_CBT_DT_ATTR_SV_LOGIC:
    case FSDB_TREE_CBT_DT_ATTR_SV_REG:
        return WP_FSDB_DATATYPE_KIND_LOGIC;
    case FSDB_TREE_CBT_DT_ATTR_BOOL:
    case FSDB_TREE_CBT_DT_ATTR_SV_BIT:
        return WP_FSDB_DATATYPE_KIND_BIT;
    case FSDB_TREE_CBT_DT_ATTR_INT32:
    case FSDB_TREE_CBT_DT_ATTR_SV_INT:
    case FSDB_TREE_CBT_DT_ATTR_SV_INTEGER:
        return WP_FSDB_DATATYPE_KIND_INT;
    case FSDB_TREE_CBT_DT_ATTR_UINT32:
    case FSDB_TREE_CBT_DT_ATTR_SV_UINT:
    case FSDB_TREE_CBT_DT_ATTR_SV_UINTEGER:
        return WP_FSDB_DATATYPE_KIND_UINT;
    case FSDB_TREE_CBT_DT_ATTR_INT64:
    case FSDB_TREE_CBT_DT_ATTR_SV_LONG_INT:
        return WP_FSDB_DATATYPE_KIND_LONG_INT;
    case FSDB_TREE_CBT_DT_ATTR_UINT64:
    case FSDB_TREE_CBT_DT_ATTR_SV_LONG_UINT:
        return WP_FSDB_DATATYPE_KIND_LONG_UINT;
    case FSDB_TREE_CBT_DT_ATTR_SV_SHORT_INT:
        return WP_FSDB_DATATYPE_KIND_SHORT_INT;
    case FSDB_TREE_CBT_DT_ATTR_SV_SHORT_UINT:
        return WP_FSDB_DATATYPE_KIND_SHORT_UINT;
    case FSDB_TREE_CBT_DT_ATTR_SV_BYTE_INT:
        return WP_FSDB_DATATYPE_KIND_BYTE;
    case FSDB_TREE_CBT_DT_ATTR_SV_BYTE_UINT:
        return WP_FSDB_DATATYPE_KIND_UBYTE;
    case FSDB_TREE_CBT_DT_FLOAT:
    case FSDB_TREE_CBT_DT_ATTR_FLOAT:
    case FSDB_TREE_CBT_DT_ATTR_DOUBLE:
    case FSDB_TREE_CBT_DT_ATTR_SV_REAL:
        return WP_FSDB_DATATYPE_KIND_REAL;
    case FSDB_TREE_CBT_DT_ATTR_SV_SHORT_REAL:
        return WP_FSDB_DATATYPE_KIND_SHORT_REAL;
    case FSDB_TREE_CBT_DT_ATTR_SV_TIME:
        return WP_FSDB_DATATYPE_KIND_TIME;
    case FSDB_TREE_CBT_DT_ATTR_STRING:
    case FSDB_TREE_CBT_DT_ATTR_SV_STRING:
    case FSDB_TREE_CBT_DT_ATTR_RAW_STRING:
        return WP_FSDB_DATATYPE_KIND_STRING;
    case FSDB_TREE_CBT_DT_ATTR_EVENT:
    case FSDB_TREE_CBT_DT_ATTR_SV_EVENT:
        return WP_FSDB_DATATYPE_KIND_EVENT;
    default:
        return WP_FSDB_DATATYPE_KIND_UNKNOWN;
    }
}

bool datatype_id_from_record(fsdbTreeCBType type, void *tree_cb_data, uint32_t *out) {
    if (out == nullptr || tree_cb_data == nullptr) {
        return false;
    }

    switch (type) {
    case FSDB_TREE_CBT_DT_ENUM: {
        auto *record = static_cast<fsdbTreeCBDataEnum *>(tree_cb_data);
        *out = static_cast<uint32_t>(record->idcode);
        return true;
    }
    case FSDB_TREE_CBT_DT_ENUM2: {
        auto *record = static_cast<fsdbTreeCBDataEnum2 *>(tree_cb_data);
        *out = static_cast<uint32_t>(record->idcode);
        return true;
    }
    case FSDB_TREE_CBT_DT_ENUM3: {
        auto *record = static_cast<fsdbTreeCBDataEnum3 *>(tree_cb_data);
        *out = static_cast<uint32_t>(record->idcode);
        return true;
    }
    case FSDB_TREE_CBT_DT_INT: {
        auto *record = static_cast<fsdbTreeCBDataInt *>(tree_cb_data);
        *out = static_cast<uint32_t>(record->idcode);
        return true;
    }
    case FSDB_TREE_CBT_DT_INT_H_N_L: {
        auto *record = static_cast<fsdbTreeCBDataIntHnL *>(tree_cb_data);
        *out = static_cast<uint32_t>(record->idcode);
        return true;
    }
    case FSDB_TREE_CBT_DT_FLOAT: {
        auto *record = static_cast<fsdbTreeCBDataFloating *>(tree_cb_data);
        *out = static_cast<uint32_t>(record->idcode);
        return true;
    }
    case FSDB_TREE_CBT_DT_ATTR_ENUM: {
        auto *record = static_cast<fsdbTreeCBDataEnumAttr *>(tree_cb_data);
        *out = static_cast<uint32_t>(record->idcode);
        return true;
    }
    case FSDB_TREE_CBT_DT_ATTR_SV_ENUM: {
        auto *record = static_cast<fsdbTreeCBDataSVEnumAttr *>(tree_cb_data);
        *out = static_cast<uint32_t>(record->idcode);
        return true;
    }
    case FSDB_TREE_CBT_DT_ATTR_LOGIC:
    case FSDB_TREE_CBT_DT_ATTR_BOOL:
    case FSDB_TREE_CBT_DT_ATTR_STRING:
    case FSDB_TREE_CBT_DT_ATTR_INT32:
    case FSDB_TREE_CBT_DT_ATTR_INT64:
    case FSDB_TREE_CBT_DT_ATTR_UINT32:
    case FSDB_TREE_CBT_DT_ATTR_UINT64:
    case FSDB_TREE_CBT_DT_ATTR_FLOAT:
    case FSDB_TREE_CBT_DT_ATTR_DOUBLE:
    case FSDB_TREE_CBT_DT_ATTR_EVENT:
    case FSDB_TREE_CBT_DT_ATTR_SV_LOGIC:
    case FSDB_TREE_CBT_DT_ATTR_SV_REG:
    case FSDB_TREE_CBT_DT_ATTR_SV_BIT:
    case FSDB_TREE_CBT_DT_ATTR_SV_LONG_INT:
    case FSDB_TREE_CBT_DT_ATTR_SV_LONG_UINT:
    case FSDB_TREE_CBT_DT_ATTR_SV_INT:
    case FSDB_TREE_CBT_DT_ATTR_SV_UINT:
    case FSDB_TREE_CBT_DT_ATTR_SV_INTEGER:
    case FSDB_TREE_CBT_DT_ATTR_SV_UINTEGER:
    case FSDB_TREE_CBT_DT_ATTR_SV_SHORT_INT:
    case FSDB_TREE_CBT_DT_ATTR_SV_SHORT_UINT:
    case FSDB_TREE_CBT_DT_ATTR_SV_BYTE_INT:
    case FSDB_TREE_CBT_DT_ATTR_SV_BYTE_UINT:
    case FSDB_TREE_CBT_DT_ATTR_SV_REAL:
    case FSDB_TREE_CBT_DT_ATTR_SV_SHORT_REAL:
    case FSDB_TREE_CBT_DT_ATTR_SV_TIME:
    case FSDB_TREE_CBT_DT_ATTR_SV_STRING:
    case FSDB_TREE_CBT_DT_ATTR_SV_EVENT:
    case FSDB_TREE_CBT_DT_ATTR_RAW_STRING: {
        auto *record = static_cast<fsdbTreeCBDataAttr *>(tree_cb_data);
        *out = static_cast<uint32_t>(record->idcode);
        return true;
    }
    default:
        return false;
    }
}

struct tree_callback_context {
    wp_fsdb_tree_callback callback;
    void *user;
    bool aborted;
};

bool_T emit_tree_event(
    tree_callback_context *context,
    wp_fsdb_tree_event event,
    const wp_fsdb_scope_record *scope,
    const wp_fsdb_signal_record *signal,
    const wp_fsdb_datatype_record *datatype
) {
    if (context == nullptr || context->callback == nullptr) {
        return static_cast<bool_T>(1);
    }
    const int rc = context->callback(
        static_cast<uint32_t>(event),
        scope,
        signal,
        datatype,
        context->user
    );
    if (rc != 0) {
        context->aborted = true;
        return static_cast<bool_T>(0);
    }
    return static_cast<bool_T>(1);
}

bool_T datatype_tree_callback(fsdbTreeCBType cb_type, void *client_data, void *tree_cb_data) {
    auto *context = static_cast<tree_callback_context *>(client_data);
    uint32_t idcode = 0;
    if (!datatype_id_from_record(cb_type, tree_cb_data, &idcode)) {
        return static_cast<bool_T>(1);
    }

    wp_fsdb_datatype_record record{};
    record.idcode = idcode;
    record.kind = static_cast<uint32_t>(map_datatype_kind(cb_type));
    return emit_tree_event(context, WP_FSDB_TREE_EVENT_DATATYPE, nullptr, nullptr, &record);
}

bool_T scope_var_tree_callback(fsdbTreeCBType cb_type, void *client_data, void *tree_cb_data) {
    auto *context = static_cast<tree_callback_context *>(client_data);
    switch (cb_type) {
    case FSDB_TREE_CBT_BEGIN_TREE:
        return emit_tree_event(context, WP_FSDB_TREE_EVENT_BEGIN_TREE, nullptr, nullptr, nullptr);
    case FSDB_TREE_CBT_SCOPE:
    case FSDB_TREE_CBT_EVENT_SCOPE:
    case FSDB_TREE_CBT_MDF_SCOPE: {
        auto *scope = static_cast<fsdbTreeCBDataScope *>(tree_cb_data);
        wp_fsdb_scope_record record{};
        record.name = scope == nullptr ? nullptr : scope->name;
        record.kind = static_cast<uint32_t>(scope == nullptr ? WP_FSDB_SCOPE_KIND_UNKNOWN : map_scope_kind(scope->type));
        record.hidden = scope != nullptr && scope->is_hidden_scope ? 1 : 0;
        return emit_tree_event(context, WP_FSDB_TREE_EVENT_SCOPE, &record, nullptr, nullptr);
    }
    case FSDB_TREE_CBT_VAR:
    case FSDB_TREE_CBT_SV_VAR:
    case FSDB_TREE_CBT_ENUM_VAR:
    case FSDB_TREE_CBT_EVENT_VAR:
    case FSDB_TREE_CBT_MDF_VAR:
    case FSDB_TREE_CBT_PACKED_VAR:
    case FSDB_TREE_CBT_PACKED_COMP_VAR: {
        auto *var = static_cast<fsdbTreeCBDataVar *>(tree_cb_data);
        wp_fsdb_signal_record record{};
        record.name = var == nullptr ? nullptr : var->name;
        record.idcode = var == nullptr ? 0 : static_cast<uint64_t>(var->u.idcode);
        record.has_bit_range = var != nullptr && var->lbitnum >= 0 && var->rbitnum >= 0 ? 1 : 0;
        record.left = var == nullptr ? 0 : static_cast<int32_t>(var->lbitnum);
        record.right = var == nullptr ? 0 : static_cast<int32_t>(var->rbitnum);
        record.has_datatype_id = var != nullptr && var->dtidcode != 0 ? 1 : 0;
        record.datatype_id = var == nullptr ? 0 : static_cast<uint32_t>(var->dtidcode);
        const wp_fsdb_signal_kind signal_kind = var == nullptr ? WP_FSDB_SIGNAL_KIND_UNKNOWN : map_signal_kind(var->type);
        record.kind = static_cast<uint32_t>(signal_kind);
        record.value_encoding = static_cast<uint32_t>(classify_value_encoding(var, signal_kind));
        return emit_tree_event(context, WP_FSDB_TREE_EVENT_SIGNAL, nullptr, &record, nullptr);
    }
    case FSDB_TREE_CBT_UPSCOPE:
    case FSDB_TREE_CBT_EVENT_UPSCOPE:
        return emit_tree_event(context, WP_FSDB_TREE_EVENT_UPSCOPE, nullptr, nullptr, nullptr);
    case FSDB_TREE_CBT_END_TREE:
    case FSDB_TREE_CBT_END_EVENT_TREE:
        return emit_tree_event(context, WP_FSDB_TREE_EVENT_END_TREE, nullptr, nullptr, nullptr);
    case FSDB_TREE_CBT_END_ALL_TREE:
        return emit_tree_event(context, WP_FSDB_TREE_EVENT_END_ALL_TREE, nullptr, nullptr, nullptr);
    default:
        return static_cast<bool_T>(1);
    }
}

void free_sample_records(wp_fsdb_sample_record *samples, std::size_t count) {
    if (samples == nullptr) {
        return;
    }
    for (std::size_t index = 0; index < count; ++index) {
        std::free(samples[index].bits);
        samples[index].bits = nullptr;
    }
    std::free(samples);
}

struct sample_records_deleter {
    std::size_t count = 0;

    void operator()(wp_fsdb_sample_record *samples) const {
        free_sample_records(samples, count);
    }
};

using owned_sample_records = std::unique_ptr<wp_fsdb_sample_record, sample_records_deleter>;

class signal_list_guard {
  public:
    explicit signal_list_guard(ffrObject *object) : object_(object) {}

    signal_list_guard(const signal_list_guard &) = delete;
    signal_list_guard &operator=(const signal_list_guard &) = delete;

    ~signal_list_guard() {
        if (object_ == nullptr) {
            return;
        }
        if (loaded_) {
            object_->ffrUnloadSignals();
        }
        if (reset_) {
            object_->ffrResetSignalList();
        }
    }

    wp_fsdb_status load(const uint64_t *idcodes, std::size_t count, char **error_message) {
        if (object_->ffrResetSignalList() != FSDB_RC_SUCCESS) {
            return fail(error_message, "FSDB Reader: failed to reset signal list before sampling");
        }
        reset_ = true;

        std::unordered_set<fsdbVarIdcode> loaded_idcodes;
        loaded_idcodes.reserve(count);
        for (std::size_t index = 0; index < count; ++index) {
            const fsdbVarIdcode idcode = static_cast<fsdbVarIdcode>(idcodes[index]);
            if (!loaded_idcodes.insert(idcode).second) {
                continue;
            }
            if (object_->ffrAddToSignalList(idcode) != FSDB_RC_SUCCESS) {
                return fail(error_message, "FSDB Reader: failed to add signal to sample list");
            }
        }

        if (object_->ffrLoadSignals() != FSDB_RC_SUCCESS) {
            return fail(error_message, "FSDB Reader: failed to load signal values");
        }
        loaded_ = true;
        return WP_FSDB_STATUS_OK;
    }

  private:
    ffrObject *object_ = nullptr;
    bool reset_ = false;
    bool loaded_ = false;
};

class vc_handle_guard {
  public:
    explicit vc_handle_guard(ffrVCTrvsHdl handle) : handle_(handle) {}

    vc_handle_guard(const vc_handle_guard &) = delete;
    vc_handle_guard &operator=(const vc_handle_guard &) = delete;

    ~vc_handle_guard() {
        if (handle_ != nullptr) {
            handle_->ffrFree();
        }
    }

    ffrVCTrvsHdl get() const {
        return handle_;
    }

    ffrVCTrvsHdl release() {
        ffrVCTrvsHdl handle = handle_;
        handle_ = nullptr;
        return handle;
    }

  private:
    ffrVCTrvsHdl handle_ = nullptr;
};

bool bit_value_to_char(byte_T value, char *out) {
    if (out == nullptr) {
        return false;
    }
    switch (value) {
    case FSDB_BT_VCD_0:
        *out = '0';
        return true;
    case FSDB_BT_VCD_1:
        *out = '1';
        return true;
    case FSDB_BT_VCD_X:
        *out = 'x';
        return true;
    case FSDB_BT_VCD_Z:
        *out = 'z';
        return true;
    default:
        return false;
    }
}

struct vc_position {
    uint64_t time_raw = 0;
    fsdbSeqNum seq_num = FSDB_EMPTY_SEQ_NUM;
};

struct sampled_bit_vector {
    bool has_value = false;
    uint64_t time_raw = 0;
    fsdbSeqNum seq_num = FSDB_EMPTY_SEQ_NUM;
    std::string bits;
};

wp_fsdb_status read_current_vc_position(
    ffrVCTrvsHdl handle,
    vc_position *position,
    char **error_message
) {
    if (handle == nullptr || position == nullptr) {
        return fail(error_message, "FSDB Reader: value-change position read received a null argument");
    }

    fsdbTag64 tag{};
    if (handle->ffrGetXTag(static_cast<void *>(&tag)) != FSDB_RC_SUCCESS) {
        return fail(error_message, "FSDB Reader: failed to read value-change time tag");
    }

    fsdbSeqNum seq_num = FSDB_EMPTY_SEQ_NUM;
    if (handle->ffrGetSeqNum(&seq_num) != FSDB_RC_SUCCESS) {
        seq_num = FSDB_EMPTY_SEQ_NUM;
    }

    position->time_raw = tag64_to_u64(tag);
    position->seq_num = seq_num;
    return WP_FSDB_STATUS_OK;
}

wp_fsdb_status decode_current_vc_bits(
    ffrVCTrvsHdl handle,
    uint_T bit_width,
    std::string *bits,
    char **error_message
) {
    if (handle == nullptr || bits == nullptr) {
        return fail(error_message, "FSDB Reader: value-change decode received a null argument");
    }

    byte_T *value_change = nullptr;
    if (handle->ffrGetVC(&value_change) != FSDB_RC_SUCCESS || value_change == nullptr) {
        return fail(error_message, "FSDB Reader: failed to read value-change data");
    }

    bits->assign(static_cast<std::size_t>(bit_width), '\0');
    for (uint_T bit_index = 0; bit_index < bit_width; ++bit_index) {
        if (!bit_value_to_char(value_change[bit_index], &(*bits)[bit_index])) {
            return fail(error_message, "FSDB Reader: signal has unsupported bit value encoding");
        }
    }
    return WP_FSDB_STATUS_OK;
}

wp_fsdb_status capture_current_bit_vector_sample(
    ffrVCTrvsHdl handle,
    uint_T bit_width,
    const vc_position &position,
    sampled_bit_vector *sample,
    char **error_message
) {
    if (sample == nullptr) {
        return fail(error_message, "FSDB Reader: sample capture received a null output");
    }

    std::string bits;
    if (decode_current_vc_bits(handle, bit_width, &bits, error_message) != WP_FSDB_STATUS_OK) {
        return WP_FSDB_STATUS_ERROR;
    }

    sample->has_value = true;
    sample->time_raw = position.time_raw;
    sample->seq_num = position.seq_num;
    sample->bits.swap(bits);
    return WP_FSDB_STATUS_OK;
}

bool position_is_not_before_selected(
    const vc_position &candidate,
    const sampled_bit_vector &selected
) {
    if (!selected.has_value) {
        return true;
    }
    if (candidate.time_raw != selected.time_raw) {
        return candidate.time_raw > selected.time_raw;
    }
    if (candidate.seq_num != FSDB_EMPTY_SEQ_NUM && selected.seq_num != FSDB_EMPTY_SEQ_NUM) {
        return candidate.seq_num >= selected.seq_num;
    }
    return true;
}

wp_fsdb_status read_final_sample_at_or_before(
    ffrVCTrvsHdl handle,
    uint64_t query_time_raw,
    uint_T bit_width,
    sampled_bit_vector *sample,
    char **error_message
) {
    if (handle == nullptr || sample == nullptr) {
        return fail(error_message, "FSDB Reader: final sample read received a null argument");
    }

    sample->has_value = false;
    sample->bits.clear();

    if (!handle->ffrHasIncoreVC()) {
        return WP_FSDB_STATUS_OK;
    }

    fsdbTag64 aligned_tag = u64_to_tag64(query_time_raw);
    int glitch_num = 0;
    if (handle->ffrGotoXTag(static_cast<void *>(&aligned_tag), &glitch_num) != FSDB_RC_SUCCESS) {
        return WP_FSDB_STATUS_OK;
    }

    vc_position position;
    if (read_current_vc_position(handle, &position, error_message) != WP_FSDB_STATUS_OK) {
        return WP_FSDB_STATUS_ERROR;
    }
    if (position.time_raw > query_time_raw) {
        return WP_FSDB_STATUS_OK;
    }

    // Ask the Reader to expose glitch metadata, then drain only records at the
    // aligned timestamp. Some dumps surface equal-time updates as separate
    // positions; sequence numbers break ties when present, and traversal order
    // is the fallback for older files. Stop as soon as time advances so normal
    // sampling stays one-position work after ffrGotoXTag().
    const uint64_t aligned_time_raw = position.time_raw;
    sampled_bit_vector selected;
    while (position.time_raw == aligned_time_raw) {
        if (position_is_not_before_selected(position, selected)) {
            sampled_bit_vector candidate;
            if (capture_current_bit_vector_sample(
                    handle,
                    bit_width,
                    position,
                    &candidate,
                    error_message
                ) != WP_FSDB_STATUS_OK) {
                return WP_FSDB_STATUS_ERROR;
            }
            selected = std::move(candidate);
        }

        if (handle->ffrGotoNextVC() != FSDB_RC_SUCCESS) {
            break;
        }
        if (read_current_vc_position(handle, &position, error_message) != WP_FSDB_STATUS_OK) {
            return WP_FSDB_STATUS_ERROR;
        }
    }

    if (selected.has_value) {
        *sample = std::move(selected);
    }
    return WP_FSDB_STATUS_OK;
}

wp_fsdb_status fill_sample_record_from_handle(
    ffrVCTrvsHdl handle,
    uint64_t idcode,
    uint64_t query_time_raw,
    wp_fsdb_sample_record *record,
    char **error_message
) {
    if (handle == nullptr || record == nullptr) {
        return fail(error_message, "FSDB Reader: sample fill received a null argument");
    }

    record->idcode = idcode;
    record->has_value = 0;
    record->bit_width = 0;
    record->value_time_raw = 0;
    record->bits = nullptr;

    const uint_T bit_width = handle->ffrGetBitSize();
    if (bit_width == 0) {
        return fail(error_message, "FSDB Reader: signal has unsupported zero-width value encoding");
    }
    record->bit_width = static_cast<uint32_t>(bit_width);

    if (handle->ffrGetBytesPerBit() != FSDB_BYTES_PER_BIT_1B) {
        return fail(error_message, "FSDB Reader: signal has unsupported non-bit-vector encoding");
    }

    sampled_bit_vector sample;
    if (read_final_sample_at_or_before(
            handle,
            query_time_raw,
            bit_width,
            &sample,
            error_message
        ) != WP_FSDB_STATUS_OK) {
        return WP_FSDB_STATUS_ERROR;
    }
    if (!sample.has_value) {
        return WP_FSDB_STATUS_OK;
    }

    char *bits = static_cast<char *>(std::malloc(sample.bits.size() + 1));
    if (bits == nullptr) {
        return fail(error_message, "FSDB Reader: failed to allocate sampled bit string");
    }
    std::memcpy(bits, sample.bits.data(), sample.bits.size());
    bits[sample.bits.size()] = '\0';

    record->has_value = 1;
    record->value_time_raw = sample.time_raw;
    record->bits = bits;
    return WP_FSDB_STATUS_OK;
}

wp_fsdb_status fill_sample_record(
    ffrObject *object,
    uint64_t idcode,
    uint64_t query_time_raw,
    wp_fsdb_sample_record *record,
    char **error_message
) {
    if (object == nullptr) {
        return fail(error_message, "FSDB Reader: sample fill received a null reader");
    }
    vc_handle_guard handle(object->ffrCreateVCTrvsHdl(static_cast<fsdbVarIdcode>(idcode)));
    if (handle.get() == nullptr) {
        return fail(error_message, "FSDB Reader: failed to create value-change traverse handle");
    }
    return fill_sample_record_from_handle(
        handle.get(),
        idcode,
        query_time_raw,
        record,
        error_message
    );
}

wp_fsdb_status append_change_times_for_handle(
    ffrVCTrvsHdl handle,
    uint64_t from_raw,
    uint64_t to_raw,
    std::vector<uint64_t> &times,
    std::unordered_set<uint64_t> &seen_times,
    char **error_message
) {
    if (handle == nullptr) {
        return fail(error_message, "FSDB Reader: change-time traversal received a null handle");
    }

    if (!handle->ffrHasIncoreVC()) {
        return WP_FSDB_STATUS_OK;
    }

    fsdbTag64 tag = u64_to_tag64(from_raw);
    if (handle->ffrGotoXTag(static_cast<void *>(&tag)) != FSDB_RC_SUCCESS) {
        if (handle->ffrGetMinXTag(static_cast<void *>(&tag)) != FSDB_RC_SUCCESS) {
            return WP_FSDB_STATUS_OK;
        }
    }

    uint64_t current = tag64_to_u64(tag);
    if (current < from_raw) {
        if (handle->ffrGotoNextVC() != FSDB_RC_SUCCESS) {
            return WP_FSDB_STATUS_OK;
        }
        if (handle->ffrGetXTag(static_cast<void *>(&tag)) != FSDB_RC_SUCCESS) {
            return fail(error_message, "FSDB Reader: failed to read value-change time tag");
        }
        current = tag64_to_u64(tag);
    }

    while (current <= to_raw) {
        if (seen_times.insert(current).second) {
            times.push_back(current);
        }
        if (handle->ffrGotoNextVC() != FSDB_RC_SUCCESS) {
            break;
        }
        if (handle->ffrGetXTag(static_cast<void *>(&tag)) != FSDB_RC_SUCCESS) {
            return fail(error_message, "FSDB Reader: failed to read next value-change time tag");
        }
        current = tag64_to_u64(tag);
    }

    return WP_FSDB_STATUS_OK;
}

wp_fsdb_status append_change_times_for_signal(
    ffrObject *object,
    uint64_t idcode,
    uint64_t from_raw,
    uint64_t to_raw,
    std::vector<uint64_t> &times,
    std::unordered_set<uint64_t> &seen_times,
    char **error_message
) {
    vc_handle_guard handle(object->ffrCreateVCTrvsHdl(static_cast<fsdbVarIdcode>(idcode)));
    if (handle.get() == nullptr) {
        return fail(error_message, "FSDB Reader: failed to create value-change traverse handle");
    }

    return append_change_times_for_handle(
        handle.get(),
        from_raw,
        to_raw,
        times,
        seen_times,
        error_message
    );
}

wp_fsdb_status copy_sorted_unique_times(
    std::vector<uint64_t> &times,
    wp_fsdb_time_list *out,
    char **error_message
) {
    std::sort(times.begin(), times.end());
    times.erase(std::unique(times.begin(), times.end()), times.end());

    out->times = nullptr;
    out->count = times.size();
    if (times.empty()) {
        return WP_FSDB_STATUS_OK;
    }

    out->times = static_cast<uint64_t *>(std::malloc(times.size() * sizeof(uint64_t)));
    if (out->times == nullptr) {
        out->count = 0;
        return fail(error_message, "FSDB Reader: failed to allocate candidate time list");
    }

    std::copy(times.begin(), times.end(), out->times);
    return WP_FSDB_STATUS_OK;
}

wp_fsdb_status read_exact_event_occurrence_from_handle(
    ffrVCTrvsHdl handle,
    uint64_t query_time_raw,
    bool &occurred,
    char **error_message
) {
    occurred = false;
    if (handle == nullptr) {
        return fail(error_message, "FSDB Reader: event traversal received a null handle");
    }

    if (!handle->ffrHasIncoreVC()) {
        return WP_FSDB_STATUS_OK;
    }

    fsdbTag64 tag = u64_to_tag64(query_time_raw);
    if (handle->ffrGotoXTag(static_cast<void *>(&tag)) != FSDB_RC_SUCCESS) {
        return WP_FSDB_STATUS_OK;
    }
    occurred = tag64_to_u64(tag) == query_time_raw;
    return WP_FSDB_STATUS_OK;
}

wp_fsdb_status read_exact_event_occurrence(
    ffrObject *object,
    uint64_t idcode,
    uint64_t query_time_raw,
    bool &occurred,
    char **error_message
) {
    vc_handle_guard handle(object->ffrCreateVCTrvsHdl(static_cast<fsdbVarIdcode>(idcode)));
    if (handle.get() == nullptr) {
        return fail(error_message, "FSDB Reader: failed to create value-change traverse handle");
    }

    return read_exact_event_occurrence_from_handle(
        handle.get(),
        query_time_raw,
        occurred,
        error_message
    );
}

constexpr std::size_t MAX_CACHED_TRAVERSE_HANDLES = 64;

struct cached_vc_handle {
    explicit cached_vc_handle(ffrVCTrvsHdl raw_handle) : handle(raw_handle) {}

    cached_vc_handle(const cached_vc_handle &) = delete;
    cached_vc_handle &operator=(const cached_vc_handle &) = delete;

    ~cached_vc_handle() {
        if (handle != nullptr) {
            handle->ffrFree();
            handle = nullptr;
        }
    }

    ffrVCTrvsHdl handle = nullptr;
    uint64_t last_used = 0;
};

}  // namespace

struct wp_fsdb_signal_session {
    explicit wp_fsdb_signal_session(wp_fsdb_reader *owner)
        : reader(owner), object(owner == nullptr ? nullptr : owner->object) {}

    wp_fsdb_signal_session(const wp_fsdb_signal_session &) = delete;
    wp_fsdb_signal_session &operator=(const wp_fsdb_signal_session &) = delete;

    ~wp_fsdb_signal_session() {
        close_noexcept();
    }

    wp_fsdb_status load(const uint64_t *idcodes, std::size_t count, char **error_message) {
        if (object == nullptr) {
            return fail(error_message, "FSDB Reader: signal session received a null reader");
        }
        if (object->ffrResetSignalList() != FSDB_RC_SUCCESS) {
            return fail(error_message, "FSDB Reader: failed to reset signal list for signal session");
        }
        reset = true;

        loaded_idcodes.reserve(count);
        for (std::size_t index = 0; index < count; ++index) {
            const fsdbVarIdcode idcode = static_cast<fsdbVarIdcode>(idcodes[index]);
            if (!loaded_idcodes.insert(idcode).second) {
                continue;
            }
            if (object->ffrAddToSignalList(idcode) != FSDB_RC_SUCCESS) {
                return fail(error_message, "FSDB Reader: failed to add signal to session list");
            }
        }

        if (!loaded_idcodes.empty()) {
            if (object->ffrLoadSignals() != FSDB_RC_SUCCESS) {
                return fail(error_message, "FSDB Reader: failed to load signal session values");
            }
            loaded = true;
        }
        return WP_FSDB_STATUS_OK;
    }

    bool has_idcode(uint64_t raw_idcode) const {
        return loaded_idcodes.find(static_cast<fsdbVarIdcode>(raw_idcode)) !=
               loaded_idcodes.end();
    }

    wp_fsdb_status require_open(char **error_message) const {
        if (object != nullptr) {
            return WP_FSDB_STATUS_OK;
        }
        return fail(error_message, "FSDB Reader: signal session is closed");
    }

    wp_fsdb_status require_idcode(uint64_t raw_idcode, char **error_message) const {
        if (has_idcode(raw_idcode)) {
            return WP_FSDB_STATUS_OK;
        }
        return fail(
            error_message,
            "FSDB Reader: idcode " + std::to_string(raw_idcode) +
                " is not loaded in the FSDB signal session"
        );
    }

    wp_fsdb_status cached_handle_for(
        uint64_t raw_idcode,
        ffrVCTrvsHdl *out,
        char **error_message
    ) {
        if (out == nullptr) {
            return fail(error_message, "FSDB Reader: cached traversal requested a null output");
        }
        *out = nullptr;
        if (require_open(error_message) != WP_FSDB_STATUS_OK) {
            return WP_FSDB_STATUS_ERROR;
        }
        if (require_idcode(raw_idcode, error_message) != WP_FSDB_STATUS_OK) {
            return WP_FSDB_STATUS_ERROR;
        }

        const fsdbVarIdcode idcode = static_cast<fsdbVarIdcode>(raw_idcode);
        const uint64_t use = ++use_counter;
        auto found = cached_handles.find(idcode);
        if (found != cached_handles.end()) {
            found->second->last_used = use;
            *out = found->second->handle;
            return WP_FSDB_STATUS_OK;
        }

        if (cached_handles.size() >= MAX_CACHED_TRAVERSE_HANDLES && !cached_handles.empty()) {
            auto evict = cached_handles.begin();
            for (auto iter = cached_handles.begin(); iter != cached_handles.end(); ++iter) {
                if (iter->second->last_used < evict->second->last_used) {
                    evict = iter;
                }
            }
            cached_handles.erase(evict);
        }

        vc_handle_guard handle(object->ffrCreateVCTrvsHdl(idcode));
        if (handle.get() == nullptr) {
            return fail(error_message, "FSDB Reader: failed to create value-change traverse handle");
        }

        auto cached = std::make_unique<cached_vc_handle>(handle.get());
        handle.release();
        cached->last_used = use;
        auto inserted = cached_handles.emplace(idcode, std::move(cached));
        *out = inserted.first->second->handle;
        return WP_FSDB_STATUS_OK;
    }

    void close_noexcept() noexcept {
        try {
            cached_handles.clear();
            if (object != nullptr) {
                if (loaded) {
                    object->ffrUnloadSignals();
                    loaded = false;
                }
                if (reset) {
                    object->ffrResetSignalList();
                    reset = false;
                }
            }
            loaded_idcodes.clear();
            if (reader != nullptr && reader->active_signal_session == this) {
                reader->active_signal_session = nullptr;
            }
            reader = nullptr;
            object = nullptr;
        } catch (...) {
            // Native close paths are best-effort; C ABI close cannot return errors.
        }
    }

    wp_fsdb_reader *reader = nullptr;
    ffrObject *object = nullptr;
    bool reset = false;
    bool loaded = false;
    uint64_t use_counter = 0;
    std::unordered_set<fsdbVarIdcode> loaded_idcodes;
    std::unordered_map<fsdbVarIdcode, std::unique_ptr<cached_vc_handle>> cached_handles;
};

extern "C" wp_fsdb_status wp_fsdb_probe(
    const char *path,
    int *is_fsdb,
    char **error_message
) {
    clear_error(error_message);
    if (path == nullptr || is_fsdb == nullptr) {
        return fail(error_message, "FSDB Reader: probe received a null argument");
    }

    try {
        std::lock_guard<std::recursive_mutex> lock(reader_mutex());
        scoped_output_suppressor output_suppressor;
        suppress_reader_messages();
        *is_fsdb = ffrObject::ffrIsFSDB(const_cast<char *>(path)) ? 1 : 0;
        return WP_FSDB_STATUS_OK;
    } catch (const std::exception &error) {
        return fail(error_message, std::string("FSDB Reader: probe failed: ") + error.what());
    } catch (...) {
        return fail_unknown_exception(error_message);
    }
}

extern "C" wp_fsdb_status wp_fsdb_open(
    const char *path,
    wp_fsdb_reader **out,
    char **error_message
) {
    clear_error(error_message);
    if (path == nullptr || out == nullptr) {
        return fail(error_message, "FSDB Reader: open received a null argument");
    }
    *out = nullptr;

    try {
        std::lock_guard<std::recursive_mutex> lock(reader_mutex());
        scoped_output_suppressor output_suppressor;
        suppress_reader_messages();
        if (!ffrObject::ffrIsFSDB(const_cast<char *>(path))) {
            return fail(error_message, "FSDB Reader: input is not an FSDB file");
        }

        ffrObject *object = ffrObject::ffrOpen3(const_cast<char *>(path));
        if (object == nullptr) {
            return fail(error_message, "FSDB Reader: ffrOpen3 failed");
        }

        wp_fsdb_reader *reader = new (std::nothrow) wp_fsdb_reader{object, nullptr};
        if (reader == nullptr) {
            object->ffrClose();
            return fail(error_message, "FSDB Reader: failed to allocate reader handle");
        }

        *out = reader;
        return WP_FSDB_STATUS_OK;
    } catch (const std::exception &error) {
        return fail(error_message, std::string("FSDB Reader: open failed: ") + error.what());
    } catch (...) {
        return fail_unknown_exception(error_message);
    }
}

extern "C" void wp_fsdb_close(wp_fsdb_reader *reader) {
    if (reader == nullptr) {
        return;
    }

    try {
        std::lock_guard<std::recursive_mutex> lock(reader_mutex());
        scoped_output_suppressor output_suppressor;
        if (reader->active_signal_session != nullptr) {
            reader->active_signal_session->close_noexcept();
        }
        if (reader->object != nullptr) {
            reader->object->ffrClose();
            reader->object = nullptr;
        }
    } catch (...) {
        // Destructors and C ABI close functions cannot report errors safely here.
    }

    delete reader;
}

extern "C" wp_fsdb_status wp_fsdb_read_metadata(
    wp_fsdb_reader *reader,
    wp_fsdb_metadata *out,
    char **error_message
) {
    clear_error(error_message);
    if (reader == nullptr || reader->object == nullptr || out == nullptr) {
        return fail(error_message, "FSDB Reader: metadata read received a null argument");
    }

    out->scale_unit = nullptr;
    out->time_start_raw = 0;
    out->time_end_raw = 0;
    out->xtag_type = 0;

    try {
        std::lock_guard<std::recursive_mutex> lock(reader_mutex());
        scoped_output_suppressor output_suppressor;
        suppress_reader_messages();

        const char *scale_unit = reader->object->ffrGetScaleUnit();
        if (scale_unit == nullptr || scale_unit[0] == '\0') {
            return fail(error_message, "FSDB Reader: scale unit metadata is empty");
        }

        owned_c_string scale_unit_copy(copy_string(scale_unit));
        if (scale_unit_copy == nullptr) {
            return fail(error_message, "FSDB Reader: failed to allocate scale unit string");
        }

        const fsdbXTagType xtag_type = reader->object->ffrGetXTagType();
        if (require_integer_time_tags(reader->object, error_message) != WP_FSDB_STATUS_OK) {
            return WP_FSDB_STATUS_ERROR;
        }

        fsdbTag64 min_tag{};
        fsdbTag64 max_tag{};
        if (reader->object->ffrGetMinFsdbTag64(&min_tag) != FSDB_RC_SUCCESS) {
            return fail(error_message, "FSDB Reader: failed to read minimum FSDB time tag");
        }
        if (reader->object->ffrGetMaxFsdbTag64(&max_tag) != FSDB_RC_SUCCESS) {
            return fail(error_message, "FSDB Reader: failed to read maximum FSDB time tag");
        }

        out->scale_unit = scale_unit_copy.release();
        out->time_start_raw = tag64_to_u64(min_tag);
        out->time_end_raw = tag64_to_u64(max_tag);
        out->xtag_type = static_cast<uint32_t>(xtag_type);
        return WP_FSDB_STATUS_OK;
    } catch (const std::exception &error) {
        return fail(
            error_message,
            std::string("FSDB Reader: metadata read failed: ") + error.what()
        );
    } catch (...) {
        return fail_unknown_exception(error_message);
    }
}

extern "C" wp_fsdb_status wp_fsdb_read_scope_var_tree(
    wp_fsdb_reader *reader,
    wp_fsdb_tree_callback callback,
    void *user,
    char **error_message
) {
    clear_error(error_message);
    if (reader == nullptr || reader->object == nullptr || callback == nullptr) {
        return fail(error_message, "FSDB Reader: hierarchy traversal received a null argument");
    }

    try {
        std::lock_guard<std::recursive_mutex> lock(reader_mutex());
        scoped_output_suppressor output_suppressor;
        suppress_reader_messages();

        tree_callback_context context{callback, user, false};
        // The Reader invokes tree callbacks synchronously while stdout/stderr are
        // suppressed process-wide. Keep the native lock held for the whole call;
        // it is recursive so a future callback-side Reader helper cannot
        // deadlock the same thread, but callback bodies should still stay small.
        if (reader->object->ffrHasDataTypeDef()) {
            uint_T block_index = 0;
            if (reader->object->ffrReadDataTypeDefByBlkIdx2(
                    datatype_tree_callback,
                    &context,
                    block_index
                ) != FSDB_RC_SUCCESS) {
                if (context.aborted) {
                    return fail(error_message, "FSDB Reader: hierarchy traversal aborted by callback");
                }
                return fail(error_message, "FSDB Reader: failed to read FSDB datatype definitions");
            }
            if (context.aborted) {
                return fail(error_message, "FSDB Reader: hierarchy traversal aborted by callback");
            }
        }

        if (reader->object->ffrReadScopeVarTree2(scope_var_tree_callback, &context) != FSDB_RC_SUCCESS) {
            if (context.aborted) {
                return fail(error_message, "FSDB Reader: hierarchy traversal aborted by callback");
            }
            return fail(error_message, "FSDB Reader: failed to read FSDB scope/variable tree");
        }
        if (context.aborted) {
            return fail(error_message, "FSDB Reader: hierarchy traversal aborted by callback");
        }

        return WP_FSDB_STATUS_OK;
    } catch (const std::exception &error) {
        return fail(
            error_message,
            std::string("FSDB Reader: hierarchy traversal failed: ") + error.what()
        );
    } catch (...) {
        return fail_unknown_exception(error_message);
    }
}

extern "C" wp_fsdb_status wp_fsdb_sample_signal_values(
    wp_fsdb_reader *reader,
    const uint64_t *idcodes,
    std::size_t count,
    uint64_t query_time_raw,
    wp_fsdb_sample_record **out,
    char **error_message
) {
    clear_error(error_message);
    if (reader == nullptr || reader->object == nullptr || out == nullptr || (count > 0 && idcodes == nullptr)) {
        return fail(error_message, "FSDB Reader: value sampling received a null argument");
    }
    *out = nullptr;

    try {
        std::lock_guard<std::recursive_mutex> lock(reader_mutex());
        scoped_output_suppressor output_suppressor;
        suppress_reader_messages();
        if (reject_active_signal_session(reader, "value sampling", error_message) !=
            WP_FSDB_STATUS_OK) {
            return WP_FSDB_STATUS_ERROR;
        }

        wp_fsdb_sample_record *raw_samples = static_cast<wp_fsdb_sample_record *>(
            std::calloc(count == 0 ? 1 : count, sizeof(wp_fsdb_sample_record))
        );
        if (raw_samples == nullptr) {
            return fail(error_message, "FSDB Reader: failed to allocate sample records");
        }
        owned_sample_records samples(raw_samples, sample_records_deleter{count});

        signal_list_guard signal_list(reader->object);
        if (signal_list.load(idcodes, count, error_message) != WP_FSDB_STATUS_OK) {
            return WP_FSDB_STATUS_ERROR;
        }

        for (std::size_t index = 0; index < count; ++index) {
            if (fill_sample_record(
                    reader->object,
                    idcodes[index],
                    query_time_raw,
                    &samples.get()[index],
                    error_message
                ) != WP_FSDB_STATUS_OK) {
                return WP_FSDB_STATUS_ERROR;
            }
        }

        *out = samples.release();
        return WP_FSDB_STATUS_OK;
    } catch (const std::exception &error) {
        return fail(
            error_message,
            std::string("FSDB Reader: value sampling failed: ") + error.what()
        );
    } catch (...) {
        return fail_unknown_exception(error_message);
    }
}

extern "C" wp_fsdb_status wp_fsdb_collect_signal_change_times(
    wp_fsdb_reader *reader,
    const uint64_t *idcodes,
    std::size_t count,
    uint64_t from_raw,
    uint64_t to_raw,
    wp_fsdb_time_list *out,
    char **error_message
) {
    clear_error(error_message);
    if (reader == nullptr || reader->object == nullptr || out == nullptr || (count > 0 && idcodes == nullptr)) {
        return fail(error_message, "FSDB Reader: candidate time collection received a null argument");
    }
    out->times = nullptr;
    out->count = 0;
    if (from_raw > to_raw || count == 0) {
        return WP_FSDB_STATUS_OK;
    }

    try {
        std::lock_guard<std::recursive_mutex> lock(reader_mutex());
        scoped_output_suppressor output_suppressor;
        suppress_reader_messages();
        if (reject_active_signal_session(reader, "candidate time collection", error_message) !=
            WP_FSDB_STATUS_OK) {
            return WP_FSDB_STATUS_ERROR;
        }

        if (require_integer_time_tags(reader->object, error_message) != WP_FSDB_STATUS_OK) {
            return WP_FSDB_STATUS_ERROR;
        }

        signal_list_guard signal_list(reader->object);
        if (signal_list.load(idcodes, count, error_message) != WP_FSDB_STATUS_OK) {
            return WP_FSDB_STATUS_ERROR;
        }

        std::vector<uint64_t> times;
        times.reserve(count);
        std::unordered_set<uint64_t> seen_times;
        seen_times.reserve(count);
        std::unordered_set<fsdbVarIdcode> visited;
        visited.reserve(count);
        for (std::size_t index = 0; index < count; ++index) {
            const fsdbVarIdcode idcode = static_cast<fsdbVarIdcode>(idcodes[index]);
            if (!visited.insert(idcode).second) {
                continue;
            }
            if (append_change_times_for_signal(
                    reader->object,
                    static_cast<uint64_t>(idcode),
                    from_raw,
                    to_raw,
                    times,
                    seen_times,
                    error_message
                ) != WP_FSDB_STATUS_OK) {
                return WP_FSDB_STATUS_ERROR;
            }
        }

        return copy_sorted_unique_times(times, out, error_message);
    } catch (const std::exception &error) {
        return fail(
            error_message,
            std::string("FSDB Reader: candidate time collection failed: ") + error.what()
        );
    } catch (...) {
        return fail_unknown_exception(error_message);
    }
}

extern "C" wp_fsdb_status wp_fsdb_signal_event_occurred(
    wp_fsdb_reader *reader,
    uint64_t idcode,
    uint64_t query_time_raw,
    int *occurred,
    char **error_message
) {
    clear_error(error_message);
    if (reader == nullptr || reader->object == nullptr || occurred == nullptr) {
        return fail(error_message, "FSDB Reader: event occurrence query received a null argument");
    }
    *occurred = 0;

    try {
        std::lock_guard<std::recursive_mutex> lock(reader_mutex());
        scoped_output_suppressor output_suppressor;
        suppress_reader_messages();
        if (reject_active_signal_session(reader, "event occurrence query", error_message) !=
            WP_FSDB_STATUS_OK) {
            return WP_FSDB_STATUS_ERROR;
        }

        if (require_integer_time_tags(reader->object, error_message) != WP_FSDB_STATUS_OK) {
            return WP_FSDB_STATUS_ERROR;
        }

        const uint64_t idcodes[] = {idcode};
        signal_list_guard signal_list(reader->object);
        if (signal_list.load(idcodes, 1, error_message) != WP_FSDB_STATUS_OK) {
            return WP_FSDB_STATUS_ERROR;
        }

        bool hit = false;
        if (read_exact_event_occurrence(
                reader->object,
                idcode,
                query_time_raw,
                hit,
                error_message
            ) != WP_FSDB_STATUS_OK) {
            return WP_FSDB_STATUS_ERROR;
        }
        *occurred = hit ? 1 : 0;
        return WP_FSDB_STATUS_OK;
    } catch (const std::exception &error) {
        return fail(
            error_message,
            std::string("FSDB Reader: event occurrence query failed: ") + error.what()
        );
    } catch (...) {
        return fail_unknown_exception(error_message);
    }
}

extern "C" wp_fsdb_status wp_fsdb_open_signal_session(
    wp_fsdb_reader *reader,
    const uint64_t *idcodes,
    std::size_t count,
    wp_fsdb_signal_session **out,
    char **error_message
) {
    clear_error(error_message);
    if (out == nullptr || reader == nullptr || reader->object == nullptr ||
        (count > 0 && idcodes == nullptr)) {
        return fail(error_message, "FSDB Reader: signal session open received a null argument");
    }
    *out = nullptr;

    try {
        std::lock_guard<std::recursive_mutex> lock(reader_mutex());
        scoped_output_suppressor output_suppressor;
        suppress_reader_messages();
        if (reader->active_signal_session != nullptr) {
            return fail(error_message, "FSDB Reader: a signal session is already active");
        }

        auto session = std::make_unique<wp_fsdb_signal_session>(reader);
        if (session->load(idcodes, count, error_message) != WP_FSDB_STATUS_OK) {
            return WP_FSDB_STATUS_ERROR;
        }

        reader->active_signal_session = session.get();
        *out = session.release();
        return WP_FSDB_STATUS_OK;
    } catch (const std::exception &error) {
        return fail(
            error_message,
            std::string("FSDB Reader: signal session open failed: ") + error.what()
        );
    } catch (...) {
        return fail_unknown_exception(error_message);
    }
}

extern "C" wp_fsdb_status wp_fsdb_signal_session_sample(
    wp_fsdb_signal_session *session,
    const uint64_t *idcodes,
    std::size_t count,
    uint64_t query_time_raw,
    wp_fsdb_sample_record **out,
    char **error_message
) {
    clear_error(error_message);
    if (session == nullptr || out == nullptr || (count > 0 && idcodes == nullptr)) {
        return fail(error_message, "FSDB Reader: signal session sampling received a null argument");
    }
    *out = nullptr;

    try {
        std::lock_guard<std::recursive_mutex> lock(reader_mutex());
        scoped_output_suppressor output_suppressor;
        suppress_reader_messages();

        wp_fsdb_sample_record *raw_samples = static_cast<wp_fsdb_sample_record *>(
            std::calloc(count == 0 ? 1 : count, sizeof(wp_fsdb_sample_record))
        );
        if (raw_samples == nullptr) {
            return fail(error_message, "FSDB Reader: failed to allocate session sample records");
        }
        owned_sample_records samples(raw_samples, sample_records_deleter{count});

        for (std::size_t index = 0; index < count; ++index) {
            ffrVCTrvsHdl handle = nullptr;
            if (session->cached_handle_for(idcodes[index], &handle, error_message) !=
                WP_FSDB_STATUS_OK) {
                return WP_FSDB_STATUS_ERROR;
            }
            if (fill_sample_record_from_handle(
                    handle,
                    idcodes[index],
                    query_time_raw,
                    &samples.get()[index],
                    error_message
                ) != WP_FSDB_STATUS_OK) {
                return WP_FSDB_STATUS_ERROR;
            }
        }

        *out = samples.release();
        return WP_FSDB_STATUS_OK;
    } catch (const std::exception &error) {
        return fail(
            error_message,
            std::string("FSDB Reader: signal session sampling failed: ") + error.what()
        );
    } catch (...) {
        return fail_unknown_exception(error_message);
    }
}

extern "C" wp_fsdb_status wp_fsdb_signal_session_collect_change_times(
    wp_fsdb_signal_session *session,
    const uint64_t *idcodes,
    std::size_t count,
    uint64_t from_raw,
    uint64_t to_raw,
    wp_fsdb_time_list *out,
    char **error_message
) {
    clear_error(error_message);
    if (session == nullptr || out == nullptr || (count > 0 && idcodes == nullptr)) {
        return fail(
            error_message,
            "FSDB Reader: signal session candidate collection received a null argument"
        );
    }
    out->times = nullptr;
    out->count = 0;
    if (from_raw > to_raw || count == 0) {
        return WP_FSDB_STATUS_OK;
    }

    try {
        std::lock_guard<std::recursive_mutex> lock(reader_mutex());
        scoped_output_suppressor output_suppressor;
        suppress_reader_messages();

        if (session->require_open(error_message) != WP_FSDB_STATUS_OK) {
            return WP_FSDB_STATUS_ERROR;
        }
        if (require_integer_time_tags(session->object, error_message) != WP_FSDB_STATUS_OK) {
            return WP_FSDB_STATUS_ERROR;
        }

        std::vector<uint64_t> times;
        times.reserve(count);
        std::unordered_set<uint64_t> seen_times;
        seen_times.reserve(count);
        std::unordered_set<fsdbVarIdcode> visited;
        visited.reserve(count);
        for (std::size_t index = 0; index < count; ++index) {
            if (session->require_idcode(idcodes[index], error_message) != WP_FSDB_STATUS_OK) {
                return WP_FSDB_STATUS_ERROR;
            }
            const fsdbVarIdcode idcode = static_cast<fsdbVarIdcode>(idcodes[index]);
            if (!visited.insert(idcode).second) {
                continue;
            }

            vc_handle_guard handle(session->object->ffrCreateVCTrvsHdl(idcode));
            if (handle.get() == nullptr) {
                return fail(error_message, "FSDB Reader: failed to create value-change traverse handle");
            }
            if (append_change_times_for_handle(
                    handle.get(),
                    from_raw,
                    to_raw,
                    times,
                    seen_times,
                    error_message
                ) != WP_FSDB_STATUS_OK) {
                return WP_FSDB_STATUS_ERROR;
            }
        }

        return copy_sorted_unique_times(times, out, error_message);
    } catch (const std::exception &error) {
        return fail(
            error_message,
            std::string("FSDB Reader: signal session candidate collection failed: ") +
                error.what()
        );
    } catch (...) {
        return fail_unknown_exception(error_message);
    }
}

extern "C" wp_fsdb_status wp_fsdb_signal_session_event_occurred(
    wp_fsdb_signal_session *session,
    uint64_t idcode,
    uint64_t query_time_raw,
    int *occurred,
    char **error_message
) {
    clear_error(error_message);
    if (session == nullptr || occurred == nullptr) {
        return fail(error_message, "FSDB Reader: signal session event query received a null argument");
    }
    *occurred = 0;

    try {
        std::lock_guard<std::recursive_mutex> lock(reader_mutex());
        scoped_output_suppressor output_suppressor;
        suppress_reader_messages();

        if (session->require_open(error_message) != WP_FSDB_STATUS_OK) {
            return WP_FSDB_STATUS_ERROR;
        }
        if (require_integer_time_tags(session->object, error_message) != WP_FSDB_STATUS_OK) {
            return WP_FSDB_STATUS_ERROR;
        }

        ffrVCTrvsHdl handle = nullptr;
        if (session->cached_handle_for(idcode, &handle, error_message) != WP_FSDB_STATUS_OK) {
            return WP_FSDB_STATUS_ERROR;
        }

        bool hit = false;
        if (read_exact_event_occurrence_from_handle(
                handle,
                query_time_raw,
                hit,
                error_message
            ) != WP_FSDB_STATUS_OK) {
            return WP_FSDB_STATUS_ERROR;
        }
        *occurred = hit ? 1 : 0;
        return WP_FSDB_STATUS_OK;
    } catch (const std::exception &error) {
        return fail(
            error_message,
            std::string("FSDB Reader: signal session event query failed: ") + error.what()
        );
    } catch (...) {
        return fail_unknown_exception(error_message);
    }
}

extern "C" void wp_fsdb_close_signal_session(wp_fsdb_signal_session *session) {
    if (session == nullptr) {
        return;
    }

    try {
        std::lock_guard<std::recursive_mutex> lock(reader_mutex());
        scoped_output_suppressor output_suppressor;
        delete session;
    } catch (...) {
        // C ABI close functions cannot report errors safely.
    }
}

extern "C" void wp_fsdb_free_samples(wp_fsdb_sample_record *samples, std::size_t count) {
    free_sample_records(samples, count);
}

extern "C" void wp_fsdb_free_time_list(wp_fsdb_time_list *list) {
    if (list == nullptr) {
        return;
    }
    std::free(list->times);
    list->times = nullptr;
    list->count = 0;
}

extern "C" void wp_fsdb_free_string(char *value) {
    std::free(value);
}

extern "C" void wp_fsdb_free_error(char *value) {
    std::free(value);
}
