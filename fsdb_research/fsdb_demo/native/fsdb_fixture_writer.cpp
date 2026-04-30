// Tiny FSDB fixture writer used only by the standalone feasibility demo.

#ifdef NOVAS_FSDB
#undef NOVAS_FSDB
#endif

#include "ffwAPI.h"

#include <cstdio>
#include <cstdlib>
#include <cstring>

namespace {

struct DigitalSignal {
    char *name;
    fsdbVarType type;
    ushort_T lbitnum;
    ushort_T rbitnum;
    fsdbVarDir direction;
    fsdbBytesPerBit bpb;
    byte_T value;
};

DigitalSignal clk = {
    const_cast<char *>("clk"),
    FSDB_VT_VCD_REG,
    0,
    0,
    FSDB_VD_IMPLICIT,
    FSDB_BYTES_PER_BIT_1B,
    static_cast<byte_T>(FSDB_BT_VCD_X),
};

void fail(const char *message) {
    std::fprintf(stderr, "error: fsdb_fixture_writer: %s\n", message);
    std::exit(1);
}

void create_scope(ffwObject *fsdb_obj) {
    ffwScopeArg2 scope;
    std::memset(&scope, 0, sizeof(scope));
    scope.size = sizeof(ffwScopeArg2);
    scope.name = const_cast<char *>("top");
    scope.type = FSDB_ST_VCD_MODULE;

    if (ffwCreateScope2(fsdb_obj, &scope) != FSDB_RC_SUCCESS) {
        fail("failed to create top scope");
    }
}

void create_signal(ffwObject *fsdb_obj) {
    ffwVarArg2 var;
    std::memset(&var, 0, sizeof(var));
    var.size = sizeof(ffwVarArg2);
    var.type = clk.type;
    var.dir = clk.direction;
    var.lbitnum = clk.lbitnum;
    var.rbitnum = clk.rbitnum;
    var.u.hdl = &clk;
    var.name = clk.name;
    var.bpb = clk.bpb;

    ffwVarMap *ret_vm = nullptr;
    if (ffwCreateVarByVarHdl2(fsdb_obj, &var, &ret_vm) != FSDB_RC_SUCCESS || ret_vm == nullptr) {
        fail("failed to create clk signal");
    }
}

void create_value(ffwObject *fsdb_obj, uint_T time, byte_T value) {
    ffw_CreateXCoorByHnL(fsdb_obj, 0, time);
    clk.value = value;
    ffw_CreateVarValueByHandle(fsdb_obj, static_cast<fsdbVarHandle>(&clk), &clk.value);
}

}  // namespace

int main(int argc, char *argv[]) {
    if (argc != 2) {
        std::fprintf(stderr, "usage: %s <output.fsdb>\n", argv[0]);
        return 2;
    }

    ffwFileArg2 file;
    std::memset(&file, 0, sizeof(file));
    file.size = sizeof(ffwFileArg2);
    file.fsdb_fname = argv[1];
    file.file_type = FSDB_FT_VERILOG;

    ffwObject *fsdb_obj = nullptr;
    if (ffwOpenFile2(&file, &fsdb_obj) != FSDB_RC_SUCCESS || fsdb_obj == nullptr) {
        fail("failed to create FSDB file");
    }

    ffw_CreateTreeByHandleScheme(fsdb_obj);
    ffw_SetScaleUnit(fsdb_obj, const_cast<char *>("1ps"));
    ffw_BeginTree(fsdb_obj);
    create_scope(fsdb_obj);
    create_signal(fsdb_obj);
    ffw_EndTree(fsdb_obj);

    create_value(fsdb_obj, 0, static_cast<byte_T>(FSDB_BT_VCD_X));
    create_value(fsdb_obj, 10, static_cast<byte_T>(FSDB_BT_VCD_0));
    create_value(fsdb_obj, 20, static_cast<byte_T>(FSDB_BT_VCD_1));
    create_value(fsdb_obj, 30, static_cast<byte_T>(FSDB_BT_VCD_Z));

    ffw_Close(fsdb_obj);
    std::printf("wrote: %s\n", argv[1]);
    return 0;
}
