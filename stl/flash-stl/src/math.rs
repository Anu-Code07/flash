//! STL math — layout, animation curves, mobile screen utilities.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MathFn {
    Abs,
    Min,
    Max,
    Clamp,
    Lerp,
    Floor,
    Ceil,
    Round,
    Sqrt,
    Pow,
    Sin,
    Cos,
    DpToPx,
    PxToDp,
    SpToPx,
}

impl MathFn {
    pub fn name(&self) -> &'static str {
        match self {
            MathFn::Abs => "abs",
            MathFn::Min => "min",
            MathFn::Max => "max",
            MathFn::Clamp => "clamp",
            MathFn::Lerp => "lerp",
            MathFn::Floor => "floor",
            MathFn::Ceil => "ceil",
            MathFn::Round => "round",
            MathFn::Sqrt => "sqrt",
            MathFn::Pow => "pow",
            MathFn::Sin => "sin",
            MathFn::Cos => "cos",
            MathFn::DpToPx => "dp_to_px",
            MathFn::PxToDp => "px_to_dp",
            MathFn::SpToPx => "sp_to_px",
        }
    }

    pub fn all() -> &'static [MathFn] {
        &[
            MathFn::Abs, MathFn::Min, MathFn::Max, MathFn::Clamp, MathFn::Lerp,
            MathFn::Floor, MathFn::Ceil, MathFn::Round, MathFn::Sqrt, MathFn::Pow,
            MathFn::Sin, MathFn::Cos, MathFn::DpToPx, MathFn::PxToDp, MathFn::SpToPx,
        ]
    }
}
