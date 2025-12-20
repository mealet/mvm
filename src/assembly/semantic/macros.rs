macro_rules! assert_arg {
    ($self:expr, $expected:literal, $expression:expr, $($pattern:tt)+) => {
        match $expression {
            $($pattern)+ => {},
            _ => {
                $self.error(AssemblyError::InvalidArgument {
                    label: format!("this expected to be {}", $expected),
                    src: $self.src.clone(),
                    span: $expression.get_span()
                });
            }
        }
    }
}

macro_rules! verify_boundary {
    ($self:expr, $value:expr, $span:expr, $type:ty) => {{
        if <$type>::try_from($value).is_err() {
            $self.error(AssemblyError::InvalidArgument {
                label: format!("value is out of `{}` bounds", stringify!($type)),
                src: $self.src.clone(),
                span: $span,
            });
        }
    }};
}

pub(crate) use assert_arg;
pub(crate) use verify_boundary;
