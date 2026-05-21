#include "wavepeek_fsdb_shim.h"

#include "ffrAPI.h"

#include <cstdlib>
#include <cstring>
#include <exception>
#include <memory>
#include <new>
#include <string>

struct wp_fsdb_reader {
    ffrObject *object;
};

namespace {

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

}  // namespace

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
        suppress_reader_messages();
        if (!ffrObject::ffrIsFSDB(const_cast<char *>(path))) {
            return fail(error_message, "FSDB Reader: input is not an FSDB file");
        }

        ffrObject *object = ffrObject::ffrOpen3(const_cast<char *>(path));
        if (object == nullptr) {
            return fail(error_message, "FSDB Reader: ffrOpen3 failed");
        }

        wp_fsdb_reader *reader = new (std::nothrow) wp_fsdb_reader{object};
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
        if (xtag_type != FSDB_XTAG_TYPE_L && xtag_type != FSDB_XTAG_TYPE_HL) {
            return fail(
                error_message,
                "FSDB Reader: unsupported non-integer FSDB time tag representation"
            );
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

extern "C" void wp_fsdb_free_string(char *value) {
    std::free(value);
}

extern "C" void wp_fsdb_free_error(char *value) {
    std::free(value);
}
