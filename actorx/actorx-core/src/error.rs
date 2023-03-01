use tea_codec::define_scope;

define_scope! {
    ActorX {
        Host;
        HostInstantiation;
        WasmCompile;
        WasmExport;
        WasmRuntime;
        GasFeeExhausted;
        PriceUndefined;
        AccessNotPermitted;
        RingInvocation;
        NativeActorCallingWasmActor;
        WasmMemoryAccess;
        NativeActorNotExists;
        ArgsTypeMismatch;
    }
}
