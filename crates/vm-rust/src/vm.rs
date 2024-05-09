use {
    crate::{ContractWrapper, VmError, VmResult, CONTRACTS},
    cw_app::{PrefixStore, QueryProvider, Vm},
    cw_types::{to_json_vec, Context},
};

pub struct RustVm {
    storage: PrefixStore,
    querier: QueryProvider<Self>,
    program: ContractWrapper,
}

impl Vm for RustVm {
    type Error = VmError;
    type Program = ContractWrapper;

    fn build_instance(
        storage: PrefixStore,
        querier: QueryProvider<Self>,
        program: Self::Program,
    ) -> VmResult<Self> {
        Ok(Self {
            storage,
            querier,
            program,
        })
    }

    fn call_in_0_out_1(&mut self, name: &str, ctx: &Context) -> VmResult<Vec<u8>> {
        match name {
            "receive" => {
                let cell = CONTRACTS; //????????????
                let contract = &cell.get().unwrap()[self.program.index];
                let res = contract.receive(ctx.clone()); // TODO: avoid this cloning by changing the trait definition
                Ok(to_json_vec(&res)?)
            },
            _ => Err(VmError::IncorrectNumberOfInputs {
                name: name.into(),
                num: 0,
            }),
        }
    }

    fn call_in_1_out_1(
        &mut self,
        name: &str,
        ctx: &Context,
        param1: impl AsRef<[u8]>,
    ) -> VmResult<Vec<u8>> {
        todo!()
    }

    fn call_in_2_out_1(
        &mut self,
        name: &str,
        ctx: &Context,
        param1: impl AsRef<[u8]>,
        param2: impl AsRef<[u8]>,
    ) -> VmResult<Vec<u8>> {
        todo!()
    }
}
