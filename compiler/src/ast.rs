/// Abstract Syntax Tree (AST) representation of Falcon code

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Function(Function),
    ExternFunction(ExternFunction),
    Struct(Struct),
    Enum(Enum),
    Module(Module),
    Import(Import),
    Const(Const),
    Impl(Impl),
    Trait(Trait),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub type_params: Vec<String>,  // Generic type parameters, e.g. <T, U>
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub is_unsafe: bool,
    pub is_pub: bool,
    pub profile: Option<String>,  // None = shared, Some("userland"|"kernel"|"baremetal") = profile-specific
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternFunction {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub is_pub: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let(LetStatement),
    Assign(AssignStatement),  // Assignment is a statement, not an expression
    Expr(Expression),
    Return(Option<Expression>),
    If(IfStatement),
    While(WhileStatement),
    For(ForStatement),
    Loop(LoopStatement),
    Match(MatchStatement),
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LetStatement {
    pub pattern: LetPattern,
    pub mutable: bool,
    pub ty: Option<Type>,
    pub value: Expression,
}

/// Pattern for let bindings - supports simple names, tuple, and struct destructuring
#[derive(Debug, Clone, PartialEq)]
pub enum LetPattern {
    Name(String),                                          // let x = ...
    Tuple(Vec<String>),                                    // let (a, b) = ...
    Struct { ty_name: String, fields: Vec<String> },       // let Point { x, y } = ...
}

/// Assignment statement: `x = value;` or `x += value;`
/// Assignment is a statement (not expression) - cannot be used as `y = (x = 5)`
#[derive(Debug, Clone, PartialEq)]
pub struct AssignStatement {
    pub target: String,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_block: Block,
    pub else_block: Option<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForStatement {
    pub var: String,
    pub iterable: Expression,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoopStatement {
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchStatement {
    pub expr: Box<Expression>,
    pub arms: Vec<MatchArm>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Variable(String),
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
    },
    Call {
        callee: Box<Expression>,
        args: Vec<Expression>,
        type_args: Vec<Type>,  // Generic type arguments, e.g. identity::<i64>(5)
    },
    MethodCall {
        receiver: Box<Expression>,
        method: String,
        args: Vec<Expression>,
    },
    StaticCall {
        type_name: String,
        method: String,
        args: Vec<Expression>,
    },
    FieldAccess {
        receiver: Box<Expression>,
        field: String,
    },
    Index {
        receiver: Box<Expression>,
        index: Box<Expression>,
    },
    Block(Block),
    If(Box<IfExpression>),
    Match(MatchStatement),
    Unsafe(Box<Expression>),
    Borrow {
        mutable: bool,
        expr: Box<Expression>,
    },
    StructLiteral {
        ty: Type,
        fields: Vec<FieldInit>,
    },
    Tuple(Vec<Expression>),
    Array(Vec<Expression>),
    Range {
        start: Option<Box<Expression>>,
        end: Option<Box<Expression>>,
    },
    Closure {
        params: Vec<Parameter>,
        return_type: Option<Type>,
        body: Box<Expression>,
    },
    // NOTE: Assignment has been moved to Statement::Assign
    // Using assignment as expression (e.g., `y = (x = 5)`) is no longer allowed
    EnumVariant {
        enum_name: String,
        variant: String,
        data: Option<Box<Expression>>,
    },
    /// The `?` operator: unwrap Ok/Some or early-return Err/None
    Try(Box<Expression>),
    /// Type cast: `expr as Type`
    Cast {
        expr: Box<Expression>,
        ty: Type,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Neg,
    Not,
    BitNot,
    Deref,
    Ref,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfExpression {
    pub condition: Box<Expression>,
    pub then_expr: Box<Expression>,
    pub else_expr: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldInit {
    pub name: String,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard,
    Literal(Literal),
    Variable(String),
    Struct {
        ty: Type,
        fields: Vec<FieldPattern>,
    },
    Tuple(Vec<Pattern>),
    EnumVariant {
        ty: Type,
        variant: String,
        data: Option<Box<Pattern>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldPattern {
    pub name: String,
    pub pattern: Pattern,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int(IntType),
    Float(FloatType),
    Bool,
    String,
    Str,
    Unit,
    Never,  // The ! type (function never returns)
    Named(String),
    TypeParam(String),  // Generic type parameter, e.g. T
    Pointer {
        mutable: bool,
        inner: Box<Type>,
    },
    Reference {
        mutable: bool,
        lifetime: Option<String>,
        inner: Box<Type>,
    },
    Array {
        inner: Box<Type>,
        size: Option<usize>,
    },
    Tuple(Vec<Type>),
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntType {
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    ISize,
    USize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatType {
    F32,
    F64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub name: String,
    pub type_params: Vec<String>,  // Generic type parameters, e.g. <T, U>
    pub fields: Vec<StructField>,
    pub is_pub: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub is_pub: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub name: String,
    pub type_params: Vec<String>,  // Generic type parameters, e.g. <T, E>
    pub variants: Vec<EnumVariant>,
    pub is_pub: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub data: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub name: String,
    pub items: Vec<Item>,
    pub is_pub: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub path: Vec<String>,
    pub alias: Option<String>,
    pub selectors: Vec<ImportSelector>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportSelector {
    pub name: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Const {
    pub name: String,
    pub ty: Type,
    pub value: Expression,
    pub is_pub: bool,
}

/// Implementation block for a type (or trait impl)
#[derive(Debug, Clone, PartialEq)]
pub struct Impl {
    pub type_name: String,          // The type we're implementing for
    pub trait_name: Option<String>,  // Some("TraitName") for `impl Trait for Type`
    pub methods: Vec<Method>,       // Methods defined in this impl
}

/// Trait definition: `trait Name { func method(self) -> Type; ... }`
#[derive(Debug, Clone, PartialEq)]
pub struct Trait {
    pub name: String,
    pub methods: Vec<TraitMethod>,
    pub is_pub: bool,
}

/// Trait method signature (no body)
#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethod {
    pub name: String,
    pub self_param: Option<SelfParam>,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
}

/// A method is like a function but with an implicit self parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Method {
    pub name: String,
    pub self_param: Option<SelfParam>,  // None = associated function (no self)
    pub params: Vec<Parameter>,          // Other parameters (excluding self)
    pub return_type: Option<Type>,
    pub body: Block,
    pub is_pub: bool,
}

/// Self parameter type for methods
#[derive(Debug, Clone, PartialEq)]
pub enum SelfParam {
    Value,      // self (takes ownership)
    Ref,        // &self (immutable borrow)
    RefMut,     // &mut self (mutable borrow)
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Int(IntType::I8) => write!(f, "i8"),
            Type::Int(IntType::I16) => write!(f, "i16"),
            Type::Int(IntType::I32) => write!(f, "i32"),
            Type::Int(IntType::I64) => write!(f, "i64"),
            Type::Int(IntType::I128) => write!(f, "i128"),
            Type::Int(IntType::U8) => write!(f, "u8"),
            Type::Int(IntType::U16) => write!(f, "u16"),
            Type::Int(IntType::U32) => write!(f, "u32"),
            Type::Int(IntType::U64) => write!(f, "u64"),
            Type::Int(IntType::U128) => write!(f, "u128"),
            Type::Int(IntType::ISize) => write!(f, "isize"),
            Type::Int(IntType::USize) => write!(f, "usize"),
            Type::Float(FloatType::F32) => write!(f, "f32"),
            Type::Float(FloatType::F64) => write!(f, "f64"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "String"),
            Type::Str => write!(f, "str"),
            Type::Unit => write!(f, "()"),
            Type::Never => write!(f, "!"),
            Type::Named(name) => write!(f, "{}", name),
            Type::TypeParam(name) => write!(f, "{}", name),
            Type::Pointer { mutable, inner } => {
                if *mutable {
                    write!(f, "*mut {}", inner)
                } else {
                    write!(f, "*const {}", inner)
                }
            }
            Type::Reference { mutable, lifetime, inner } => {
                if let Some(lt) = lifetime {
                    write!(f, "&'{}", lt)?;
                } else {
                    write!(f, "&")?;
                }
                if *mutable {
                    write!(f, "mut ")?;
                }
                write!(f, "{}", inner)
            }
            Type::Array { inner, size } => {
                if let Some(s) = size {
                    write!(f, "[{}; {}]", inner, s)
                } else {
                    write!(f, "[{}]", inner)
                }
            }
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", ty)?;
                }
                write!(f, ")")
            }
            Type::Function { params, return_type } => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", return_type)
            }
        }
    }
}
