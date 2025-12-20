use miette::SourceSpan;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    SectionDef {
        id: String,
        span: SourceSpan,
    },

    EntryDef {
        label: String,
        span: SourceSpan,
    },

    LabelDef {
        id: String,
        span: SourceSpan,
    },

    Directive {
        directive: String,
        args: Vec<Expression>,
        span: SourceSpan,
    },

    ComptimeExpr {
        expr: Box<Expression>,
        span: SourceSpan,
    },

    Instruction {
        name: String,
        args: Vec<Expression>,
        span: SourceSpan,
    },

    BinaryExpr {
        op: String,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
        span: SourceSpan,
    },

    UIntConstant(u64, SourceSpan),
    StringConstant(String, SourceSpan),

    AsmConstant(String, SourceSpan),
    AsmReg(String, SourceSpan),

    LabelRef(String, SourceSpan),
    CurrentPtr(SourceSpan),

    None,
}

impl Expression {
    pub fn get_span(&self) -> SourceSpan {
        match self {
            Expression::SectionDef { span, .. } => *span,
            Expression::EntryDef { span, .. } => *span,
            Expression::LabelDef { span, .. } => *span,
            Expression::Directive { span, .. } => *span,
            Expression::ComptimeExpr { span, .. } => *span,
            Expression::Instruction { span, .. } => *span,
            Expression::BinaryExpr { span, .. } => *span,
            Expression::UIntConstant(_, span) => *span,
            Expression::StringConstant(_, span) => *span,
            Expression::AsmConstant(_, span) => *span,
            Expression::AsmReg(_, span) => *span,
            Expression::LabelRef(_, span) => *span,
            Expression::CurrentPtr(span) => *span,
            Expression::None => (0, 0).into(),
        }
    }
}
