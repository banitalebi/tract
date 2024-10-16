use crate::tensor::MetalTensorExt;
use crate::{kernels, MetalTensor};
use derive_new::new;
use std::fmt::Debug;
use tract_core::internal::*;

#[derive(Debug, Clone, new, Hash)]
pub struct MetalMultiBroadcastTo {
    pub shape: ShapeFact,
}

impl Op for MetalMultiBroadcastTo {
    fn name(&self) -> Cow<str> {
        "MetalMultiBroadcastTo".into()
    }

    op_as_typed_op!();
}

impl EvalOp for MetalMultiBroadcastTo {
    fn is_stateless(&self) -> bool {
        true
    }

    fn eval_with_session(
        &self,
        session: &SessionState,
        inputs: TVec<TValue>,
    ) -> TractResult<TVec<TValue>> {
        let shape = self.shape.eval_to_usize(&session.resolved_symbols)?;
        objc::rc::autoreleasepool(|| {
            crate::METAL_CONTEXT.with_borrow(|context| {
                let opaque = args_1!(inputs);
                let input = opaque.to_metal_tensor()?;
                let output = unsafe { MetalTensor::uninitialized_dt(input.datum_type(), &shape)? };
                kernels::array::MultiBroadcast.dispatch_eval(context, input, 0, &output)?;
                Ok(tvec![output.into_opaque_tensor().into_tvalue()])
            })
        })
    }
}

impl TypedOp for MetalMultiBroadcastTo {
    fn output_facts(&self, inputs: &[&TypedFact]) -> TractResult<TVec<TypedFact>> {
        crate::utils::metal_tmp_output_facts(inputs, |facts| {
            let mut fact = facts[0].datum_type.fact(self.shape.clone());
            fact.uniform.clone_from(&inputs[0].uniform);
            Ok(tvec!(fact))
        })
        .with_context(|| anyhow::anyhow!("Error while computing facts for {:?}", self.name()))
    }

    as_op!();
}
