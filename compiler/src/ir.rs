/// Falcon Intermediate Representation (IR) v0.1
/// 
/// The IR is versioned and stable. This is v0.1, the initial version.
/// Future versions will be additive only (v0.2, v0.3, etc.)

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IrModule {
    pub version: String, // "0.1"
    pub imports: Vec<IrImportResolution>,
    pub functions: Vec<IrFunction>,
    pub extern_functions: Vec<String>,  // names of extern func declarations
    pub types: Vec<IrType>,
    pub structs: Vec<IrStructDef>,
    pub enums: Vec<IrEnumDef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrImportResolution {
    pub module: String,
    pub resolved_to: String,
    pub profile: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IrStructDef {
    pub name: String,
    pub fields: Vec<(String, IrType)>,
}

/// Enum definition in IR - defines all possible states
/// Tag is explicit, making variant discrimination deterministic
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IrEnumDef {
    pub name: String,
    pub variants: Vec<IrEnumVariant>,
}

/// Enum variant with explicit tag and optional payload
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IrEnumVariant {
    pub name: String,
    pub tag: u32,                    // Discriminant value (0, 1, 2, ...)
    pub payload: Option<IrType>,     // Payload type if any (e.g., Failed(Error))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IrFunction {
    pub name: String,
    pub params: Vec<IrParameter>,
    pub return_type: Option<IrType>,
    pub body: IrBlock,
    pub is_unsafe: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IrParameter {
    pub name: String,
    pub ty: IrType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IrBlock {
    pub instructions: Vec<IrInstruction>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IrInstruction {
    // Memory operations
    Alloc {
        ty: IrType,
        size: usize,
        dest: IrValue,
    },
    Load {
        addr: IrValue,
        dest: IrValue,
    },
    Store {
        addr: IrValue,
        value: IrValue,
    },
    VolatileLoad {
        addr: IrValue,
        dest: IrValue,
    },
    VolatileStore {
        addr: IrValue,
        value: IrValue,
    },
    Free {
        addr: IrValue,
    },
    
    // Ownership operations
    Move {
        src: IrValue,
        dest: IrValue,
    },
    BorrowImm {
        src: IrValue,
        dest: IrValue,
        lifetime: String,
    },
    BorrowMut {
        src: IrValue,
        dest: IrValue,
        lifetime: String,
    },
    Drop {
        value: IrValue,
    },
    Copy {
        src: IrValue,
        dest: IrValue,
    },
    
    // Control flow
    Branch {
        label: String,
    },
    BranchCond {
        condition: IrValue,
        true_label: String,
        false_label: String,
    },
    Call {
        func: String,
        args: Vec<IrValue>,
        dest: Option<IrValue>,
    },
    Return {
        value: Option<IrValue>,
    },
    Label {
        name: String,
    },
    
    // Arithmetic
    Add {
        left: IrValue,
        right: IrValue,
        dest: IrValue,
    },
    Sub {
        left: IrValue,
        right: IrValue,
        dest: IrValue,
    },
    Mul {
        left: IrValue,
        right: IrValue,
        dest: IrValue,
    },
    Div {
        left: IrValue,
        right: IrValue,
        dest: IrValue,
    },
    Mod {
        left: IrValue,
        right: IrValue,
        dest: IrValue,
    },
    
    // Comparison
    Eq {
        left: IrValue,
        right: IrValue,
        dest: IrValue,
    },
    Ne {
        left: IrValue,
        right: IrValue,
        dest: IrValue,
    },
    Lt {
        left: IrValue,
        right: IrValue,
        dest: IrValue,
    },
    Le {
        left: IrValue,
        right: IrValue,
        dest: IrValue,
    },
    Gt {
        left: IrValue,
        right: IrValue,
        dest: IrValue,
    },
    Ge {
        left: IrValue,
        right: IrValue,
        dest: IrValue,
    },
    
    // Logical
    And {
        left: IrValue,
        right: IrValue,
        dest: IrValue,
    },
    Or {
        left: IrValue,
        right: IrValue,
        dest: IrValue,
    },
    Not {
        operand: IrValue,
        dest: IrValue,
    },
    
    // Bitwise
    BitAnd {
        left: IrValue,
        right: IrValue,
        dest: IrValue,
    },
    BitOr {
        left: IrValue,
        right: IrValue,
        dest: IrValue,
    },
    BitXor {
        left: IrValue,
        right: IrValue,
        dest: IrValue,
    },
    Shl {
        left: IrValue,
        right: IrValue,
        dest: IrValue,
    },
    Shr {
        left: IrValue,
        right: IrValue,
        dest: IrValue,
    },
    
    // Unary
    Neg {
        operand: IrValue,
        dest: IrValue,
    },
    BitNot {
        operand: IrValue,
        dest: IrValue,
    },
    /// Take the address of a variable (produces a pointer)
    AddrOf {
        operand: IrValue,
        dest: IrValue,
    },
    /// Dereference a pointer (load from pointer)
    PtrDeref {
        operand: IrValue,
        dest: IrValue,
    },
    
    // Literals
    Literal {
        value: IrLiteral,
        dest: IrValue,
    },
    
    /// Type hint for variables — tells codegen the declared type for correct alloca sizing
    TypeHint {
        name: String,
        ty: IrType,
    },
    
    // Index/Field access
    Index {
        base: IrValue,
        index: IrValue,
        dest: IrValue,
    },
    FieldAccess {
        base: IrValue,
        field: String,
        dest: IrValue,
    },
    StructInit {
        struct_name: String,
        fields: Vec<(String, IrValue)>,
        dest: IrValue,
    },
    
    // Error handling (explicit in IR)
    Panic {
        message: String,
    },
    Unwrap {
        value: IrValue,
        dest: IrValue,
        message: Option<String>,
    },
    
    // Allocation intent (explicit in IR)
    StackAlloc {
        ty: IrType,
        size: usize,
        dest: IrValue,
    },
    HeapAlloc {
        ty: IrType,
        size: usize,
        dest: IrValue,
    },
    
    // Profile-specific operations
    BoundsCheck {
        value: IrValue,
        bound: IrValue,
        on_fail: String, // Label to jump to on failure
    },
    
    // Range creation
    Range {
        start: IrValue,
        end: IrValue,
        dest: IrValue,
    },
    
    // Array initialization
    ArrayInit {
        elements: Vec<IrValue>,
        dest: IrValue,
    },
    
    // Enum operations
    /// Create an enum value with a specific variant
    EnumInit {
        enum_name: String,
        variant: String,
        payload: Option<IrValue>,
        dest: IrValue,
    },
    /// Extract the tag (discriminant) from an enum value
    EnumTag {
        value: IrValue,
        dest: IrValue,
    },
    /// Extract the payload from an enum variant (unsafe if wrong variant)
    EnumPayload {
        value: IrValue,
        variant: String,
        dest: IrValue,
    },
    
    // Closure operations
    /// Create a closure value (function pointer + captured environment)
    ClosureCreate {
        closure_id: usize,           // Unique ID for this closure
        params: Vec<IrParameter>,    // Closure parameters
        body: Vec<IrInstruction>,    // Closure body instructions
        captures: Vec<String>,       // Names of captured variables
        dest: IrValue,
    },
    /// Call a closure
    ClosureCall {
        closure: IrValue,            // The closure value
        args: Vec<IrValue>,          // Arguments passed to closure
        dest: Option<IrValue>,       // Result destination
    },
    
    /// Type cast: `expr as Type`
    /// At runtime, generates trunc/zext/sext/inttoptr/ptrtoint as needed
    Cast {
        operand: IrValue,
        target_type: IrType,
        dest: IrValue,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IrValue {
    Variable(String),
    Temporary(usize),
    Constant(IrLiteral),
}

impl IrValue {
    /// Get the name of this value as a string
    pub fn name(&self) -> String {
        match self {
            IrValue::Variable(name) => name.clone(),
            IrValue::Temporary(id) => format!("_temp_{}", id),
            IrValue::Constant(_) => "_const".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IrLiteral {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IrType {
    Int(IntType),
    Float(FloatType),
    Bool,
    String,
    Str,
    Unit,
    Named(String),
    Pointer {
        mutable: bool,
        inner: Box<IrType>,
    },
    Reference {
        mutable: bool,
        lifetime: Option<String>,
        inner: Box<IrType>,
    },
    Array {
        inner: Box<IrType>,
        size: Option<usize>,
    },
    Tuple(Vec<IrType>),
    Function {
        params: Vec<IrType>,
        return_type: Box<IrType>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FloatType {
    F32,
    F64,
}

/// Loop context for tracking break/continue labels in nested loops
struct LoopContext {
    break_label: String,
    continue_label: String,
}

/// Context for IR generation (tracks temporaries, variables, enums, loops, etc.)
struct IrContext {
    temp_counter: usize,
    variables: std::collections::HashMap<String, IrValue>,
    /// Type names for variables (for method call resolution)
    variable_types: std::collections::HashMap<String, String>, // var_name -> type_name
    /// Names (variables/temporaries) that currently hold closure values
    closure_values: std::collections::HashSet<String>,
    /// Enum registry for looking up variant tags
    enum_registry: std::collections::HashMap<String, Vec<(String, u32)>>, // enum_name -> [(variant_name, tag)]
    /// Loop context stack for break/continue label resolution
    loop_stack: Vec<LoopContext>,
    /// Const values for compile-time inlining
    const_values: std::collections::HashMap<String, IrLiteral>,
    /// Variables declared with `let mut` — reassignment allowed
    mutable_vars: std::collections::HashSet<String>,
}

impl IrContext {
    fn new() -> Self {
        Self {
            temp_counter: 0,
            variables: std::collections::HashMap::new(),
            variable_types: std::collections::HashMap::new(),
            closure_values: std::collections::HashSet::new(),
            enum_registry: std::collections::HashMap::new(),
            loop_stack: Vec::new(),
            const_values: std::collections::HashMap::new(),
            mutable_vars: std::collections::HashSet::new(),
        }
    }
    
    fn next_temp(&mut self) -> IrValue {
        let temp = IrValue::Temporary(self.temp_counter);
        self.temp_counter += 1;
        temp
    }
    
    fn add_variable(&mut self, name: String, value: IrValue) {
        self.variables.insert(name, value);
    }
    
    fn get_variable(&self, name: &str) -> Option<IrValue> {
        self.variables.get(name).cloned()
    }
    
    /// Set the type name for a variable (used for method call resolution)
    fn set_variable_type(&mut self, name: String, type_name: String) {
        self.variable_types.insert(name, type_name);
    }
    
    /// Get the type name for a variable
    fn get_variable_type(&self, name: &str) -> Option<&String> {
        self.variable_types.get(name)
    }

    /// Mark whether a value currently represents a closure
    fn set_value_is_closure(&mut self, value: &IrValue, is_closure: bool) {
        let name = value.name();
        if is_closure {
            self.closure_values.insert(name);
        } else {
            self.closure_values.remove(&name);
        }
    }

    /// Check whether a value is known to represent a closure
    fn is_closure_value(&self, value: &IrValue) -> bool {
        self.closure_values.contains(&value.name())
    }

    /// Check whether a named variable currently represents a closure
    fn is_closure_variable(&self, name: &str) -> bool {
        self.closure_values.contains(name)
    }
    
    /// Register an enum with its variants and tags
    fn register_enum(&mut self, name: String, variants: Vec<(String, u32)>) {
        self.enum_registry.insert(name, variants);
    }

    /// Check whether an enum with this name is registered
    fn has_enum(&self, enum_name: &str) -> bool {
        self.enum_registry.contains_key(enum_name)
    }

    /// Check whether an enum has a particular variant
    fn enum_has_variant(&self, enum_name: &str, variant_name: &str) -> bool {
        self.enum_registry
            .get(enum_name)
            .map(|variants| variants.iter().any(|(name, _)| name == variant_name))
            .unwrap_or(false)
    }
    
    /// Look up tag for a variant scoped to a specific enum type
    fn get_variant_tag_scoped(&self, enum_name: &str, variant_name: &str) -> Option<u32> {
        if let Some(variants) = self.enum_registry.get(enum_name) {
            for (name, tag) in variants {
                if name == variant_name {
                    return Some(*tag);
                }
            }
        }
        None
    }
    
    /// Look up tag for a variant by name (searches all enums — fallback for unscoped contexts)
    fn get_variant_tag(&self, variant_name: &str) -> Option<u32> {
        for (_enum_name, variants) in &self.enum_registry {
            for (name, tag) in variants {
                if name == variant_name {
                    return Some(*tag);
                }
            }
        }
        None
    }
    
    /// Push a new loop context (called when entering while/for/loop)
    fn push_loop(&mut self, break_label: String, continue_label: String) {
        self.loop_stack.push(LoopContext { break_label, continue_label });
    }
    
    /// Pop the current loop context (called when exiting while/for/loop)
    fn pop_loop(&mut self) {
        self.loop_stack.pop();
    }
    
    /// Get the current break label (for break statement)
    fn current_break_label(&self) -> Option<&str> {
        self.loop_stack.last().map(|ctx| ctx.break_label.as_str())
    }
    
    /// Get the current continue label (for continue statement)
    fn current_continue_label(&self) -> Option<&str> {
        self.loop_stack.last().map(|ctx| ctx.continue_label.as_str())
    }
}

/// Helper to generate tag value from variant name
/// TODO: Replace with proper lookup from enum registry for accurate tagging
fn variant_name_to_tag(variant: &str) -> u32 {
    // For now, use a simple hash-based approach
    // In production, this should look up the actual tag from IrModule.enums
    let mut hash: u32 = 0;
    for b in variant.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(b as u32);
    }
    hash % 1000 // Keep it small and readable
}

/// Convert AST to IR
pub fn ast_to_ir(ast: &crate::ast::Program) -> Result<IrModule, String> {
    let mut module = IrModule {
        version: "0.1".to_string(),
        imports: Vec::new(),
        functions: Vec::new(),
        extern_functions: Vec::new(),
        types: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
    };
    
    // FIRST PASS: Collect all enums and consts so we can look up variant tags and inline const values
    let mut enum_registry: std::collections::HashMap<String, Vec<(String, u32)>> = std::collections::HashMap::new();
    let mut const_table: std::collections::HashMap<String, IrLiteral> = std::collections::HashMap::new();
    
    for item in &ast.items {
        match item {
            crate::ast::Item::Enum(e) => {
                // Convert AST enum to IR with explicit tags
                let variants: Vec<IrEnumVariant> = e.variants.iter()
                    .enumerate()
                    .map(|(i, v)| IrEnumVariant {
                        name: v.name.clone(),
                        tag: i as u32,
                        payload: v.data.as_ref().map(ast_type_to_ir),
                    })
                    .collect();
                
                // Register for tag lookup
                let variant_tags: Vec<(String, u32)> = variants.iter()
                    .map(|v| (v.name.clone(), v.tag))
                    .collect();
                enum_registry.insert(e.name.clone(), variant_tags);
                
                module.enums.push(IrEnumDef {
                    name: e.name.clone(),
                    variants,
                });
            }
            crate::ast::Item::Const(c) => {
                // Inline const values — evaluate with access to already-defined consts
                if let Some(lit) = ast_expr_to_literal_with_consts(&c.value, Some(&const_table)) {
                    const_table.insert(c.name.clone(), lit);
                }
            }
            _ => {}
        }
    }
    
    // SECOND PASS: Convert functions and structs (with enum registry available)
    for item in &ast.items {
        match item {
            crate::ast::Item::Function(func) => {
                module.functions.push(ast_function_to_ir_with_enums(func, &enum_registry, &const_table)?);
            }
            crate::ast::Item::Struct(s) => {
                module.structs.push(IrStructDef {
                    name: s.name.clone(),
                    fields: s.fields.iter().map(|f| (f.name.clone(), ast_type_to_ir(&f.ty))).collect(),
                });
            }
            crate::ast::Item::ExternFunction(ef) => {
                // Declaration-only item. Store name for call validation.
                module.extern_functions.push(ef.name.clone());
            }
            crate::ast::Item::Enum(_) => {
                // Already handled in first pass
            }
            crate::ast::Item::Impl(impl_block) => {
                // Convert methods to IR functions with mangled names
                for method in &impl_block.methods {
                    let ir_func = ast_method_to_ir(method, &impl_block.type_name, &enum_registry, &const_table)?;
                    module.functions.push(ir_func);
                }
            }
            _ => {
                // Other items handled later
            }
        }
    }
    
    Ok(module)
}

/// Convert a const expression to an IR literal (compile-time evaluation)
fn ast_expr_to_literal(expr: &crate::ast::Expression) -> Option<IrLiteral> {
    ast_expr_to_literal_with_consts(expr, None)
}

fn ast_expr_to_literal_with_consts(
    expr: &crate::ast::Expression,
    const_table: Option<&std::collections::HashMap<String, IrLiteral>>,
) -> Option<IrLiteral> {
    match expr {
        crate::ast::Expression::Literal(lit) => match lit {
            crate::ast::Literal::Int(v) => Some(IrLiteral::Int(*v)),
            crate::ast::Literal::Float(v) => Some(IrLiteral::Float(*v)),
            crate::ast::Literal::Bool(v) => Some(IrLiteral::Bool(*v)),
            crate::ast::Literal::String(v) => Some(IrLiteral::String(v.clone())),
            crate::ast::Literal::Char(v) => Some(IrLiteral::Char(*v)),
        },
        crate::ast::Expression::Variable(name) => {
            // Resolve references to other consts
            const_table.and_then(|ct| ct.get(name).cloned())
        }
        crate::ast::Expression::BinaryOp { op, left, right } => {
            let l = ast_expr_to_literal_with_consts(left, const_table)?;
            let r = ast_expr_to_literal_with_consts(right, const_table)?;
            match (&l, &r) {
                (IrLiteral::Int(a), IrLiteral::Int(b)) => {
                    let result = match op {
                        crate::ast::BinaryOperator::Add => a.wrapping_add(*b),
                        crate::ast::BinaryOperator::Sub => a.wrapping_sub(*b),
                        crate::ast::BinaryOperator::Mul => a.wrapping_mul(*b),
                        crate::ast::BinaryOperator::Div => {
                            if *b == 0 { return None; }
                            a.wrapping_div(*b)
                        }
                        crate::ast::BinaryOperator::Mod => {
                            if *b == 0 { return None; }
                            a.wrapping_rem(*b)
                        }
                        crate::ast::BinaryOperator::Shl => a.wrapping_shl(*b as u32),
                        crate::ast::BinaryOperator::Shr => a.wrapping_shr(*b as u32),
                        crate::ast::BinaryOperator::BitAnd => a & b,
                        crate::ast::BinaryOperator::BitOr => a | b,
                        crate::ast::BinaryOperator::BitXor => a ^ b,
                        _ => return None,
                    };
                    Some(IrLiteral::Int(result))
                }
                (IrLiteral::Float(a), IrLiteral::Float(b)) => {
                    let result = match op {
                        crate::ast::BinaryOperator::Add => a + b,
                        crate::ast::BinaryOperator::Sub => a - b,
                        crate::ast::BinaryOperator::Mul => a * b,
                        crate::ast::BinaryOperator::Div => a / b,
                        _ => return None,
                    };
                    Some(IrLiteral::Float(result))
                }
                _ => None,
            }
        }
        crate::ast::Expression::UnaryOp { op, operand } => {
            let val = ast_expr_to_literal_with_consts(operand, const_table)?;
            match (&val, op) {
                (IrLiteral::Int(n), crate::ast::UnaryOperator::Neg) => Some(IrLiteral::Int(-n)),
                (IrLiteral::Int(n), crate::ast::UnaryOperator::BitNot) => Some(IrLiteral::Int(!n)),
                (IrLiteral::Bool(b), crate::ast::UnaryOperator::Not) => Some(IrLiteral::Bool(!b)),
                (IrLiteral::Float(f), crate::ast::UnaryOperator::Neg) => Some(IrLiteral::Float(-f)),
                _ => None,
            }
        }
        // Cast expression: unwrap to inner value (type cast is a no-op for const eval)
        crate::ast::Expression::Cast { expr, .. } => {
            ast_expr_to_literal_with_consts(expr, const_table)
        }
        _ => None,
    }
}

fn ast_function_to_ir_with_enums(
    func: &crate::ast::Function,
    enum_registry: &std::collections::HashMap<String, Vec<(String, u32)>>,
    const_table: &std::collections::HashMap<String, IrLiteral>,
) -> Result<IrFunction, String> {
    let mut ctx = IrContext::new();
    
    // Register all enums in context for tag lookup
    for (enum_name, variants) in enum_registry {
        ctx.register_enum(enum_name.clone(), variants.clone());
    }
    
    // Register all const values for inlining
    for (name, value) in const_table {
        ctx.const_values.insert(name.clone(), value.clone());
    }
    
    let mut instructions = Vec::new();
    
    // Add parameters to context - use actual parameter names
    for param in func.params.iter() {
        let param_value = IrValue::Variable(param.name.clone());
        ctx.add_variable(param.name.clone(), param_value.clone());
        ctx.mutable_vars.insert(param.name.clone()); // params are implicitly mutable
    }
    
    // Inject const values as Literal instructions at function start
    // This registers them as regular variables so all Variable handlers find them
    for (name, lit) in &ctx.const_values.clone() {
        let dest = ctx.next_temp();
        instructions.push(IrInstruction::Literal {
            value: lit.clone(),
            dest: dest.clone(),
        });
        ctx.add_variable(name.clone(), dest);
    }
    
    // Convert function body
    for stmt in &func.body.statements {
        let stmt_insts = ast_stmt_to_ir(stmt, &mut ctx)?;
        instructions.extend(stmt_insts);
    }
    
    Ok(IrFunction {
        name: func.name.clone(),
        params: func.params.iter().map(|p| IrParameter {
            name: p.name.clone(),
            ty: ast_type_to_ir(&p.ty),
        }).collect(),
        return_type: func.return_type.as_ref().map(ast_type_to_ir),
        body: IrBlock { instructions },
        is_unsafe: func.is_unsafe,
    })
}

/// Convert a method to an IR function with mangled name
fn ast_method_to_ir(
    method: &crate::ast::Method,
    type_name: &str,
    enum_registry: &std::collections::HashMap<String, Vec<(String, u32)>>,
    const_table: &std::collections::HashMap<String, IrLiteral>,
) -> Result<IrFunction, String> {
    let mut ctx = IrContext::new();
    
    // Register all enums in context for tag lookup
    for (enum_name, variants) in enum_registry {
        ctx.register_enum(enum_name.clone(), variants.clone());
    }
    
    // Register all const values for inlining
    for (name, value) in const_table {
        ctx.const_values.insert(name.clone(), value.clone());
    }
    
    // Mangle the function name: TypeName__methodName
    let mangled_name = format!("{}_{}", type_name, method.name);
    
    let mut params = Vec::new();
    
    // If method has self parameter, add it as first parameter (pointer to struct)
    if method.self_param.is_some() {
        let self_param = IrParameter {
            name: "self".to_string(),
            ty: IrType::Pointer { mutable: false, inner: Box::new(IrType::Named(type_name.to_string())) },
        };
        params.push(self_param);
        // Register self in context
        ctx.add_variable("self".to_string(), IrValue::Variable("self".to_string()));
    }
    
    // Add other parameters
    for param in method.params.iter() {
        params.push(IrParameter {
            name: param.name.clone(),
            ty: ast_type_to_ir(&param.ty),
        });
        ctx.add_variable(param.name.clone(), IrValue::Variable(param.name.clone()));
        ctx.mutable_vars.insert(param.name.clone()); // params are implicitly mutable
    }
    
    let mut instructions = Vec::new();
    
    // Inject const values as Literal instructions at method start
    for (name, lit) in &ctx.const_values.clone() {
        let dest = ctx.next_temp();
        instructions.push(IrInstruction::Literal {
            value: lit.clone(),
            dest: dest.clone(),
        });
        ctx.add_variable(name.clone(), dest);
    }
    
    // Convert method body
    for stmt in &method.body.statements {
        let stmt_insts = ast_stmt_to_ir(stmt, &mut ctx)?;
        instructions.extend(stmt_insts);
    }
    
    Ok(IrFunction {
        name: mangled_name,
        params,
        return_type: method.return_type.as_ref().map(ast_type_to_ir),
        body: IrBlock { instructions },
        is_unsafe: false,
    })
}

fn ast_stmt_to_ir(stmt: &crate::ast::Statement, ctx: &mut IrContext) -> Result<Vec<IrInstruction>, String> {
    // Helper to get value from an expression
    fn get_expr_value(expr: &crate::ast::Expression, insts: &mut Vec<IrInstruction>, ctx: &mut IrContext) -> Result<IrValue, String> {
        match expr {
            crate::ast::Expression::Variable(name) => {
                // Check const values first (compile-time inlining)
                if let Some(lit) = ctx.const_values.get(name).cloned() {
                    let dest = ctx.next_temp();
                    insts.push(IrInstruction::Literal {
                        value: lit,
                        dest: dest.clone(),
                    });
                    return Ok(dest);
                }
                ctx.get_variable(name)
                    .ok_or_else(|| format!("Undefined variable: {}", name))
            }
            _ => {
                let expr_insts = ast_expr_to_ir(expr, ctx)?;
                let start_idx = insts.len();
                insts.extend(expr_insts);
                
                if let Some(last) = insts.get(start_idx..).and_then(|s| s.last()) {
                    Ok(match last {
                        IrInstruction::Literal { dest, .. } => dest.clone(),
                        IrInstruction::Add { dest, .. } => dest.clone(),
                        IrInstruction::Sub { dest, .. } => dest.clone(),
                        IrInstruction::Mul { dest, .. } => dest.clone(),
                        IrInstruction::Div { dest, .. } => dest.clone(),
                        IrInstruction::Mod { dest, .. } => dest.clone(),
                        IrInstruction::Eq { dest, .. } => dest.clone(),
                        IrInstruction::Ne { dest, .. } => dest.clone(),
                        IrInstruction::Lt { dest, .. } => dest.clone(),
                        IrInstruction::Le { dest, .. } => dest.clone(),
                        IrInstruction::Gt { dest, .. } => dest.clone(),
                        IrInstruction::Ge { dest, .. } => dest.clone(),
                        IrInstruction::And { dest, .. } => dest.clone(),
                        IrInstruction::Or { dest, .. } => dest.clone(),
                        IrInstruction::Not { dest, .. } => dest.clone(),
                        IrInstruction::BitNot { dest, .. } => dest.clone(),
                        IrInstruction::BitAnd { dest, .. } => dest.clone(),
                        IrInstruction::BitOr { dest, .. } => dest.clone(),
                        IrInstruction::BitXor { dest, .. } => dest.clone(),
                        IrInstruction::Shl { dest, .. } => dest.clone(),
                        IrInstruction::Shr { dest, .. } => dest.clone(),
                        IrInstruction::Neg { dest, .. } => dest.clone(),
                        IrInstruction::Call { dest, .. } |
                        IrInstruction::ClosureCall { dest, .. } => dest.clone().unwrap_or_else(|| ctx.next_temp()),
                        IrInstruction::Move { dest, .. } => dest.clone(),
                        IrInstruction::Index { dest, .. } => dest.clone(),
                        IrInstruction::FieldAccess { dest, .. } => dest.clone(),
                        IrInstruction::BorrowImm { dest, .. } => dest.clone(),
                        IrInstruction::BorrowMut { dest, .. } => dest.clone(),
                        IrInstruction::Range { dest, .. } => dest.clone(),
                        IrInstruction::StructInit { dest, .. } => dest.clone(),
                        IrInstruction::ArrayInit { dest, .. } => dest.clone(),
                        IrInstruction::VolatileLoad { dest, .. } => dest.clone(),
                        IrInstruction::EnumInit { dest, .. } => dest.clone(),
                        IrInstruction::EnumTag { dest, .. } => dest.clone(),
                        IrInstruction::EnumPayload { dest, .. } => dest.clone(),
                        IrInstruction::ClosureCreate { dest, .. } => dest.clone(),
                        _ => ctx.next_temp(),
                    })
                } else {
                    Ok(ctx.next_temp())
                }
            }
        }
    }
    
    match stmt {
        crate::ast::Statement::Let(let_stmt) => {
            let mut insts = Vec::new();
            
            // Get value from expression (handles variables properly)
            let source = get_expr_value(&let_stmt.value, &mut insts, ctx)?;
            
            match &let_stmt.pattern {
                crate::ast::LetPattern::Name(name) => {
                    // Simple binding: let x = expr;
                    let var_value = IrValue::Variable(name.clone());
                    ctx.add_variable(name.clone(), var_value.clone());
                    
                    // Track mutability
                    if let_stmt.mutable {
                        ctx.mutable_vars.insert(name.clone());
                    }
                    
                    // Track type for struct literals (needed for method call resolution)
                    if let crate::ast::Expression::StructLiteral { ty, .. } = &let_stmt.value {
                        if let crate::ast::Type::Named(type_name) = ty {
                            ctx.set_variable_type(name.clone(), type_name.clone());
                        }
                    }

                    let source_is_closure = ctx.is_closure_value(&source);
                    
                    // Emit type hint if Let has explicit type annotation
                    // This tells codegen to use the correct LLVM type (e.g. i8 for u8)
                    if let Some(ref ty) = let_stmt.ty {
                        insts.push(IrInstruction::TypeHint {
                            name: name.clone(),
                            ty: ast_type_to_ir(ty),
                        });
                    }
                    
                    // Add move instruction
                    insts.push(IrInstruction::Move {
                        src: source,
                        dest: var_value,
                    });

                    // Track whether this binding now holds a closure value
                    ctx.set_value_is_closure(
                        &IrValue::Variable(name.clone()),
                        source_is_closure,
                    );
                }
                crate::ast::LetPattern::Tuple(names) => {
                    // Tuple destructuring: let (a, b, c) = expr;
                    // Move RHS to a temp first
                    let tuple_temp = ctx.next_temp();
                    insts.push(IrInstruction::Move {
                        src: source,
                        dest: tuple_temp.clone(),
                    });
                    
                    // Extract each field by index
                    for (i, name) in names.iter().enumerate() {
                        let field_dest = IrValue::Variable(name.clone());
                        ctx.add_variable(name.clone(), field_dest.clone());
                        
                        let index_temp = ctx.next_temp();
                        insts.push(IrInstruction::Index {
                            base: tuple_temp.clone(),
                            index: IrValue::Constant(IrLiteral::Int(i as i64)),
                            dest: index_temp.clone(),
                        });
                        insts.push(IrInstruction::Move {
                            src: index_temp,
                            dest: field_dest,
                        });
                    }
                }
                crate::ast::LetPattern::Struct { ty_name: _, fields } => {
                    // Struct destructuring: let Point { x, y } = expr;
                    // Move RHS to a temp first
                    let struct_temp = ctx.next_temp();
                    insts.push(IrInstruction::Move {
                        src: source,
                        dest: struct_temp.clone(),
                    });
                    
                    // Extract each field by name
                    for field_name in fields {
                        let field_dest = IrValue::Variable(field_name.clone());
                        ctx.add_variable(field_name.clone(), field_dest.clone());
                        
                        let access_temp = ctx.next_temp();
                        insts.push(IrInstruction::FieldAccess {
                            base: struct_temp.clone(),
                            field: field_name.clone(),
                            dest: access_temp.clone(),
                        });
                        insts.push(IrInstruction::Move {
                            src: access_temp,
                            dest: field_dest,
                        });
                    }
                }
            }
            
            Ok(insts)
        }
        crate::ast::Statement::Assign(assign_stmt) => {
            // Assignment statement: x = value;
            let mut insts = Vec::new();
            
            // Evaluate the RHS
            let source = get_expr_value(&assign_stmt.value, &mut insts, ctx)?;
            let source_is_closure = ctx.is_closure_value(&source);
            
            // Check mutability — reject reassignment of immutable variables
            if !ctx.mutable_vars.contains(&assign_stmt.target)
                && ctx.get_variable(&assign_stmt.target).is_some() 
            {
                return Err(format!(
                    "Cannot assign to immutable variable '{}'. Declare with `let mut {}` to allow reassignment.",
                    assign_stmt.target, assign_stmt.target
                ));
            }
            
            // Move to the target variable
            let dest = IrValue::Variable(assign_stmt.target.clone());
            insts.push(IrInstruction::Move { src: source, dest });

            // Assignment can change whether the target is a closure binding
            ctx.set_value_is_closure(
                &IrValue::Variable(assign_stmt.target.clone()),
                source_is_closure,
            );
            
            Ok(insts)
        }
        crate::ast::Statement::Return(expr) => {
            let mut insts = Vec::new();
            
            let return_value = if let Some(e) = expr {
                Some(get_expr_value(e, &mut insts, ctx)?)
            } else {
                None
            };
            
            insts.push(IrInstruction::Return {
                value: return_value,
            });
            Ok(insts)
        }
        crate::ast::Statement::Expr(expr) => {
            ast_expr_to_ir(expr, ctx)
        }
        crate::ast::Statement::If(if_stmt) => {
            let mut insts = Vec::new();
            let cond_insts = ast_expr_to_ir(&if_stmt.condition, ctx)?;
            insts.extend(cond_insts);
            
            // Get condition result
            let cond_temp = if let Some(last) = insts.last() {
                match last {
                    IrInstruction::Literal { dest, .. } => dest.clone(),
                    IrInstruction::Eq { dest, .. } => dest.clone(),
                    IrInstruction::Ne { dest, .. } => dest.clone(),
                    IrInstruction::Lt { dest, .. } => dest.clone(),
                    IrInstruction::Le { dest, .. } => dest.clone(),
                    IrInstruction::Gt { dest, .. } => dest.clone(),
                    IrInstruction::Ge { dest, .. } => dest.clone(),
                    IrInstruction::And { dest, .. } => dest.clone(),
                    IrInstruction::Or { dest, .. } => dest.clone(),
                    IrInstruction::Not { dest, .. } => dest.clone(),
                    IrInstruction::BitNot { dest, .. } => dest.clone(),
                    IrInstruction::BitAnd { dest, .. } => dest.clone(),
                    IrInstruction::BitOr { dest, .. } => dest.clone(),
                    IrInstruction::BitXor { dest, .. } => dest.clone(),
                    IrInstruction::Shl { dest, .. } => dest.clone(),
                    IrInstruction::Shr { dest, .. } => dest.clone(),
                    _ => ctx.next_temp(),
                }
            } else {
                ctx.next_temp()
            };
            
            let true_label = format!("if_true_{}", ctx.temp_counter);
            let false_label = format!("if_false_{}", ctx.temp_counter);
            let end_label = format!("if_end_{}", ctx.temp_counter);
            
            insts.push(IrInstruction::BranchCond {
                condition: cond_temp,
                true_label: true_label.clone(),
                false_label: false_label.clone(),
            });
            
            // True branch
            insts.push(IrInstruction::Label { name: true_label });
            for stmt in &if_stmt.then_block.statements {
                insts.extend(ast_stmt_to_ir(stmt, ctx)?);
            }
            insts.push(IrInstruction::Branch { label: end_label.clone() });
            
            // False branch (if exists)
            if let Some(else_block) = &if_stmt.else_block {
                insts.push(IrInstruction::Label { name: false_label });
                for stmt in &else_block.statements {
                    insts.extend(ast_stmt_to_ir(stmt, ctx)?);
                }
            } else {
                insts.push(IrInstruction::Label { name: false_label });
            }
            insts.push(IrInstruction::Branch { label: end_label.clone() });
            
            insts.push(IrInstruction::Label { name: end_label });
            Ok(insts)
        }
        crate::ast::Statement::While(while_stmt) => {
            let mut insts = Vec::new();
            let loop_start = format!("loop_start_{}", ctx.temp_counter);
            let loop_body = format!("loop_body_{}", ctx.temp_counter);
            let loop_end = format!("loop_end_{}", ctx.temp_counter);
            
            // Push loop context for break/continue resolution
            ctx.push_loop(loop_end.clone(), loop_start.clone());
            
            insts.push(IrInstruction::Label { name: loop_start.clone() });
            
            let cond_insts = ast_expr_to_ir(&while_stmt.condition, ctx)?;
            insts.extend(cond_insts);
            
            let cond_temp = if let Some(last) = insts.last() {
                match last {
                    IrInstruction::Literal { dest, .. } => dest.clone(),
                    IrInstruction::Eq { dest, .. } => dest.clone(),
                    IrInstruction::Ne { dest, .. } => dest.clone(),
                    IrInstruction::Lt { dest, .. } => dest.clone(),
                    IrInstruction::Le { dest, .. } => dest.clone(),
                    IrInstruction::Gt { dest, .. } => dest.clone(),
                    IrInstruction::Ge { dest, .. } => dest.clone(),
                    IrInstruction::And { dest, .. } => dest.clone(),
                    IrInstruction::Or { dest, .. } => dest.clone(),
                    IrInstruction::Not { dest, .. } => dest.clone(),
                    IrInstruction::BitNot { dest, .. } => dest.clone(),
                    _ => ctx.next_temp(),
                }
            } else {
                ctx.next_temp()
            };
            
            insts.push(IrInstruction::BranchCond {
                condition: cond_temp,
                true_label: loop_body.clone(),
                false_label: loop_end.clone(),
            });
            
            insts.push(IrInstruction::Label { name: loop_body });
            for stmt in &while_stmt.body.statements {
                insts.extend(ast_stmt_to_ir(stmt, ctx)?);
            }
            insts.push(IrInstruction::Branch { label: loop_start });
            
            insts.push(IrInstruction::Label { name: loop_end });
            
            // Pop loop context
            ctx.pop_loop();
            
            Ok(insts)
        }
        crate::ast::Statement::Break => {
            // Use loop context stack for proper label
            match ctx.current_break_label() {
                Some(label) => Ok(vec![IrInstruction::Branch { label: label.to_string() }]),
                None => Err("break statement outside of loop".to_string()),
            }
        }
        crate::ast::Statement::Continue => {
            // Use loop context stack for proper label
            match ctx.current_continue_label() {
                Some(label) => Ok(vec![IrInstruction::Branch { label: label.to_string() }]),
                None => Err("continue statement outside of loop".to_string()),
            }
        }
        crate::ast::Statement::For(for_stmt) => {
            let mut insts = Vec::new();
            let loop_id = ctx.temp_counter;
            let loop_start = format!("for_start_{}", loop_id);
            let loop_body = format!("for_body_{}", loop_id);
            let loop_end = format!("for_end_{}", loop_id);
            
            // Push loop context for break/continue resolution
            ctx.push_loop(loop_end.clone(), loop_start.clone());
            
            match &for_stmt.iterable {
                crate::ast::Expression::Range { start, end } => {
                    // === Range-based for loop (counted) ===
                    let start_expr = start.as_ref().map(|e| e.as_ref().clone())
                        .unwrap_or(crate::ast::Expression::Literal(crate::ast::Literal::Int(0)));
                    let end_expr = end.as_ref().map(|e| e.as_ref().clone())
                        .ok_or("For loop range must have an end value")?;
                    
                    // Initialize loop variable with start value
                    let start_val = get_expr_value(&start_expr, &mut insts, ctx)?;
                    let loop_var = IrValue::Variable(for_stmt.var.clone());
                    ctx.add_variable(for_stmt.var.clone(), loop_var.clone());
                    ctx.mutable_vars.insert(for_stmt.var.clone()); // loop vars are implicitly mutable
                    insts.push(IrInstruction::Move {
                        src: start_val,
                        dest: loop_var.clone(),
                    });
                    
                    // Get end value
                    let end_val = get_expr_value(&end_expr, &mut insts, ctx)?;
                    
                    // Loop start label
                    insts.push(IrInstruction::Label { name: loop_start.clone() });
                    
                    // Condition: loop_var < end_val
                    let cond_dest = ctx.next_temp();
                    insts.push(IrInstruction::Lt {
                        left: loop_var.clone(),
                        right: end_val,
                        dest: cond_dest.clone(),
                    });
                    
                    // Branch based on condition
                    insts.push(IrInstruction::BranchCond {
                        condition: cond_dest,
                        true_label: loop_body.clone(),
                        false_label: loop_end.clone(),
                    });
                    
                    // Loop body
                    insts.push(IrInstruction::Label { name: loop_body });
                    for stmt in &for_stmt.body.statements {
                        insts.extend(ast_stmt_to_ir(stmt, ctx)?);
                    }
                    
                    // Increment loop variable: loop_var = loop_var + 1
                    let one = ctx.next_temp();
                    insts.push(IrInstruction::Literal {
                        value: IrLiteral::Int(1),
                        dest: one.clone(),
                    });
                    let new_val = ctx.next_temp();
                    insts.push(IrInstruction::Add {
                        left: loop_var.clone(),
                        right: one,
                        dest: new_val.clone(),
                    });
                    insts.push(IrInstruction::Move {
                        src: new_val,
                        dest: loop_var,
                    });
                    
                    // Branch back to loop start
                    insts.push(IrInstruction::Branch { label: loop_start });
                }
                _ => {
                    // === Iterator/Array-based for loop (index-based desugaring) ===
                    // Desugars: for x in collection { body }
                    // Into:    let __arr = collection;
                    //          let __len = len(__arr);
                    //          let __idx = 0;
                    //          while __idx < __len {
                    //              let x = __arr[__idx];
                    //              body;
                    //              __idx += 1;
                    //          }
                    
                    // Evaluate the iterable expression
                    let arr_val = get_expr_value(&for_stmt.iterable, &mut insts, ctx)?;
                    let arr_var = ctx.next_temp();
                    insts.push(IrInstruction::Move {
                        src: arr_val,
                        dest: arr_var.clone(),
                    });
                    
                    // Get array length via Call to falcon_array_len or use known length
                    // For now, store length as a temp that gets resolved at runtime
                    let len_var = ctx.next_temp();
                    insts.push(IrInstruction::Call {
                        func: "falcon_array_len".to_string(),
                        args: vec![arr_var.clone()],
                        dest: Some(len_var.clone()),
                    });
                    
                    // Initialize index to 0
                    let idx_var = IrValue::Variable(format!("__iter_idx_{}", loop_id));
                    let zero = ctx.next_temp();
                    insts.push(IrInstruction::Literal {
                        value: IrLiteral::Int(0),
                        dest: zero.clone(),
                    });
                    insts.push(IrInstruction::Move {
                        src: zero,
                        dest: idx_var.clone(),
                    });
                    
                    // Loop start
                    insts.push(IrInstruction::Label { name: loop_start.clone() });
                    
                    // Condition: idx < len
                    let cond_dest = ctx.next_temp();
                    insts.push(IrInstruction::Lt {
                        left: idx_var.clone(),
                        right: len_var.clone(),
                        dest: cond_dest.clone(),
                    });
                    
                    insts.push(IrInstruction::BranchCond {
                        condition: cond_dest,
                        true_label: loop_body.clone(),
                        false_label: loop_end.clone(),
                    });
                    
                    // Loop body: let x = arr[idx]
                    insts.push(IrInstruction::Label { name: loop_body });
                    
                    let loop_var = IrValue::Variable(for_stmt.var.clone());
                    ctx.add_variable(for_stmt.var.clone(), loop_var.clone());
                    ctx.mutable_vars.insert(for_stmt.var.clone()); // loop vars are implicitly mutable
                    let element = ctx.next_temp();
                    insts.push(IrInstruction::Index {
                        base: arr_var.clone(),
                        index: idx_var.clone(),
                        dest: element.clone(),
                    });
                    insts.push(IrInstruction::Move {
                        src: element,
                        dest: loop_var,
                    });
                    
                    // User body
                    for stmt in &for_stmt.body.statements {
                        insts.extend(ast_stmt_to_ir(stmt, ctx)?);
                    }
                    
                    // Increment index
                    let one = ctx.next_temp();
                    insts.push(IrInstruction::Literal {
                        value: IrLiteral::Int(1),
                        dest: one.clone(),
                    });
                    let new_idx = ctx.next_temp();
                    insts.push(IrInstruction::Add {
                        left: idx_var.clone(),
                        right: one,
                        dest: new_idx.clone(),
                    });
                    insts.push(IrInstruction::Move {
                        src: new_idx,
                        dest: idx_var,
                    });
                    
                    insts.push(IrInstruction::Branch { label: loop_start });
                }
            }
            
            // Loop end label
            insts.push(IrInstruction::Label { name: loop_end });
            
            // Pop loop context
            ctx.pop_loop();
            
            Ok(insts)
        }
        crate::ast::Statement::Loop(loop_stmt) => {
            let mut insts = Vec::new();
            let loop_id = ctx.temp_counter;
            let loop_start = format!("loop_start_{}", loop_id);
            let loop_end = format!("loop_end_{}", loop_id);
            
            // Push loop context for break/continue resolution
            ctx.push_loop(loop_end.clone(), loop_start.clone());
            
            insts.push(IrInstruction::Label { name: loop_start.clone() });
            
            for stmt in &loop_stmt.body.statements {
                insts.extend(ast_stmt_to_ir(stmt, ctx)?);
            }
            
            insts.push(IrInstruction::Branch { label: loop_start });
            
            // End label (for break)
            insts.push(IrInstruction::Label { name: loop_end });
            
            // Pop loop context
            ctx.pop_loop();
            
            Ok(insts)
        }
        crate::ast::Statement::Match(match_stmt) => {
            let mut insts = Vec::new();
            let match_id = ctx.temp_counter;
            ctx.temp_counter += 1; // Ensure unique IDs for nested matches
            let match_end = format!("match_end_{}", match_id);
            
            // Evaluate the match expression
            let match_val = get_expr_value(&match_stmt.expr, &mut insts, ctx)?;
            
            // Generate code for each arm
            for (i, arm) in match_stmt.arms.iter().enumerate() {
                let arm_body_label = format!("match_body_{}_{}", match_id, i);
                let next_check_label = format!("match_check_{}_{}", match_id, i + 1);
                
                match &arm.pattern {
                    crate::ast::Pattern::Wildcard => {
                        // Wildcard always matches - just execute the body
                        insts.push(IrInstruction::Label { name: arm_body_label });
                        let arm_insts = ast_expr_to_ir(&arm.body, ctx)?;
                        insts.extend(arm_insts);
                        insts.push(IrInstruction::Branch { label: match_end.clone() });
                    }
                    crate::ast::Pattern::Literal(lit) => {
                        // Compare match value with literal
                        let lit_val = ctx.next_temp();
                        let ir_lit = match lit {
                            crate::ast::Literal::Int(n) => IrLiteral::Int(*n),
                            crate::ast::Literal::Float(f) => IrLiteral::Float(*f),
                            crate::ast::Literal::Bool(b) => IrLiteral::Bool(*b),
                            crate::ast::Literal::String(s) => IrLiteral::String(s.clone()),
                            crate::ast::Literal::Char(c) => IrLiteral::Char(*c),
                        };
                        insts.push(IrInstruction::Literal {
                            value: ir_lit,
                            dest: lit_val.clone(),
                        });
                        
                        let cond = ctx.next_temp();
                        insts.push(IrInstruction::Eq {
                            left: match_val.clone(),
                            right: lit_val,
                            dest: cond.clone(),
                        });
                        
                        insts.push(IrInstruction::BranchCond {
                            condition: cond,
                            true_label: arm_body_label.clone(),
                            false_label: next_check_label.clone(),
                        });
                        
                        insts.push(IrInstruction::Label { name: arm_body_label });
                        let arm_insts = ast_expr_to_ir(&arm.body, ctx)?;
                        insts.extend(arm_insts);
                        insts.push(IrInstruction::Branch { label: match_end.clone() });
                        
                        insts.push(IrInstruction::Label { name: next_check_label });
                    }
                    crate::ast::Pattern::Variable(name) => {
                        // Bind the match value to the variable
                        let var_val = IrValue::Variable(name.clone());
                        ctx.add_variable(name.clone(), var_val.clone());
                        insts.push(IrInstruction::Move {
                            src: match_val.clone(),
                            dest: var_val,
                        });
                        
                        let arm_insts = ast_expr_to_ir(&arm.body, ctx)?;
                        insts.extend(arm_insts);
                        insts.push(IrInstruction::Branch { label: match_end.clone() });
                    }
                    crate::ast::Pattern::EnumVariant { ty, variant, data } => {
                        // Match on enum variant
                        // Get enum name from type
                        let enum_name = match ty {
                            crate::ast::Type::Named(n) => n.clone(),
                            _ => return Err("Enum pattern must have named type".to_string()),
                        };
                        
                        // Extract tag from match value
                        let tag_val = ctx.next_temp();
                        insts.push(IrInstruction::EnumTag {
                            value: match_val.clone(),
                            dest: tag_val.clone(),
                        });
                        
                        // Get expected tag for this variant, scoped by enum type
                        let expected_tag_value = ctx.get_variant_tag_scoped(&enum_name, variant)
                            .or_else(|| ctx.get_variant_tag(variant))
                            .ok_or_else(|| format!("Unknown enum variant in match: {}::{}", enum_name, variant))? as i64;
                        let expected_tag = ctx.next_temp();
                        insts.push(IrInstruction::Literal {
                            value: IrLiteral::Int(expected_tag_value as i64),
                            dest: expected_tag.clone(),
                        });
                        
                        // Compare tags
                        let cond = ctx.next_temp();
                        insts.push(IrInstruction::Eq {
                            left: tag_val,
                            right: expected_tag,
                            dest: cond.clone(),
                        });
                        
                        insts.push(IrInstruction::BranchCond {
                            condition: cond,
                            true_label: arm_body_label.clone(),
                            false_label: next_check_label.clone(),
                        });
                        
                        insts.push(IrInstruction::Label { name: arm_body_label });
                        
                        // Bind payload if present
                        if let Some(payload_pattern) = data {
                            let payload_val = ctx.next_temp();
                            insts.push(IrInstruction::EnumPayload {
                                value: match_val.clone(),
                                variant: variant.clone(),
                                dest: payload_val.clone(),
                            });
                            
                            // Handle the payload pattern (typically a variable binding)
                            if let crate::ast::Pattern::Variable(name) = payload_pattern.as_ref() {
                                let var_val = IrValue::Variable(name.clone());
                                ctx.add_variable(name.clone(), var_val.clone());
                                insts.push(IrInstruction::Move {
                                    src: payload_val,
                                    dest: var_val,
                                });
                            }
                            // Else: wildcard or other patterns - payload ignored
                        }
                        
                        let arm_insts = ast_expr_to_ir(&arm.body, ctx)?;
                        insts.extend(arm_insts);
                        insts.push(IrInstruction::Branch { label: match_end.clone() });
                        
                        insts.push(IrInstruction::Label { name: next_check_label });
                    }
                    crate::ast::Pattern::Struct { .. } | crate::ast::Pattern::Tuple(_) => {
                        // TODO: Implement struct and tuple patterns
                        return Err("Struct and tuple pattern matching not yet implemented".to_string());
                    }
                }
            }
            
            insts.push(IrInstruction::Label { name: match_end });
            Ok(insts)
        }
    }
}

fn expression_kind(expr: &crate::ast::Expression) -> &'static str {
    match expr {
        crate::ast::Expression::Literal(_) => "Literal",
        crate::ast::Expression::Variable(_) => "Variable",
        crate::ast::Expression::BinaryOp { .. } => "BinaryOp",
        crate::ast::Expression::UnaryOp { .. } => "UnaryOp",
        crate::ast::Expression::Call { .. } => "Call",
        crate::ast::Expression::MethodCall { .. } => "MethodCall",
        crate::ast::Expression::StaticCall { .. } => "StaticCall",
        crate::ast::Expression::FieldAccess { .. } => "FieldAccess",
        crate::ast::Expression::Index { .. } => "Index",
        crate::ast::Expression::Block(_) => "Block",
        crate::ast::Expression::If(_) => "If",
        crate::ast::Expression::Match(_) => "Match",
        crate::ast::Expression::Unsafe(_) => "Unsafe",
        crate::ast::Expression::Borrow { .. } => "Borrow",
        crate::ast::Expression::StructLiteral { .. } => "StructLiteral",
        crate::ast::Expression::Tuple(_) => "Tuple",
        crate::ast::Expression::Array(_) => "Array",
        crate::ast::Expression::Range { .. } => "Range",
        crate::ast::Expression::Closure { .. } => "Closure",
        crate::ast::Expression::EnumVariant { .. } => "EnumVariant",
        crate::ast::Expression::Try(_) => "Try",
        crate::ast::Expression::Cast { .. } => "Cast",
    }
}

/// Collect free variables in an expression that aren't in the given parameter set.
/// Used for closure capture detection.
fn collect_free_variables(
    expr: &crate::ast::Expression,
    params: &std::collections::HashSet<String>,
    captured: &mut Vec<String>,
) {
    use crate::ast::Expression;
    match expr {
        Expression::Variable(name) => {
            if !params.contains(name) && !captured.contains(name) {
                captured.push(name.clone());
            }
        }
        Expression::BinaryOp { left, right, .. } => {
            collect_free_variables(left, params, captured);
            collect_free_variables(right, params, captured);
        }
        Expression::UnaryOp { operand, .. } => {
            collect_free_variables(operand, params, captured);
        }
        Expression::Call { callee, args, .. } => {
            collect_free_variables(callee, params, captured);
            for arg in args {
                collect_free_variables(arg, params, captured);
            }
        }
        Expression::Block(block) => {
            for stmt in &block.statements {
                collect_free_variables_stmt(stmt, params, captured);
            }
        }
        Expression::If(if_expr) => {
            collect_free_variables(&if_expr.condition, params, captured);
            collect_free_variables(&if_expr.then_expr, params, captured);
            collect_free_variables(&if_expr.else_expr, params, captured);
        }
        Expression::FieldAccess { receiver, .. } => {
            collect_free_variables(receiver, params, captured);
        }
        Expression::Index { receiver, index } => {
            collect_free_variables(receiver, params, captured);
            collect_free_variables(index, params, captured);
        }
        Expression::Closure { params: inner_params, body, .. } => {
            // Inner closure params shadow outer — add them to skip set
            let mut inner_set = params.clone();
            for p in inner_params {
                inner_set.insert(p.name.clone());
            }
            collect_free_variables(body, &inner_set, captured);
        }
        // Literals, etc. don't reference variables
        _ => {}
    }
}

fn collect_free_variables_stmt(
    stmt: &crate::ast::Statement,
    params: &std::collections::HashSet<String>,
    captured: &mut Vec<String>,
) {
    use crate::ast::Statement;
    match stmt {
        Statement::Let(let_stmt) => {
            collect_free_variables(&let_stmt.value, params, captured);
        }
        Statement::Assign(assign) => {
            collect_free_variables(&assign.value, params, captured);
        }
        Statement::Expr(expr) | Statement::Return(Some(expr)) => {
            collect_free_variables(expr, params, captured);
        }
        _ => {}
    }
}

fn ast_expr_to_ir(expr: &crate::ast::Expression, ctx: &mut IrContext) -> Result<Vec<IrInstruction>, String> {

    match expr {
        crate::ast::Expression::Literal(lit) => {
            let ir_lit = match lit {
                crate::ast::Literal::Int(n) => IrLiteral::Int(*n),
                crate::ast::Literal::Float(f) => IrLiteral::Float(*f),
                crate::ast::Literal::Bool(b) => IrLiteral::Bool(*b),
                crate::ast::Literal::String(s) => IrLiteral::String(s.clone()),
                crate::ast::Literal::Char(c) => IrLiteral::Char(*c),
            };
            let dest = ctx.next_temp();
            Ok(vec![IrInstruction::Literal {
                value: ir_lit,
                dest,
            }])
        }
        crate::ast::Expression::Variable(name) => {
            if ctx.get_variable(name).is_some() {
                // Variable exists, return it
                Ok(Vec::new()) // Variable reference, no instruction needed
            } else if let Some(lit) = ctx.const_values.get(name).cloned() {
                // Const value — inline as a literal instruction
                let dest = ctx.next_temp();
                Ok(vec![IrInstruction::Literal {
                    value: lit,
                    dest,
                }])
            } else {
                Err(format!("Undefined variable: {}", name))
            }
        }
        crate::ast::Expression::BinaryOp { op, left, right } => {
            let mut insts = Vec::new();
            
            // Helper to get the value from an expression (may be a variable or computed value)
            fn get_expr_value(expr: &crate::ast::Expression, insts: &mut Vec<IrInstruction>, ctx: &mut IrContext) -> Result<IrValue, String> {
                match expr {
                    crate::ast::Expression::Variable(name) => {
                        // Check const values first (compile-time inlining)
                        if let Some(lit) = ctx.const_values.get(name).cloned() {
                            let dest = ctx.next_temp();
                            insts.push(IrInstruction::Literal {
                                value: lit,
                                dest: dest.clone(),
                            });
                            return Ok(dest);
                        }
                        // Variable reference - return the variable directly
                        ctx.get_variable(name)
                            .ok_or_else(|| format!("Undefined variable: {}", name))
                    }
                    _ => {
                        // Evaluate expression and get result from last instruction
                        let expr_insts = ast_expr_to_ir(expr, ctx)?;
                        let start_idx = insts.len();
                        insts.extend(expr_insts);
                        
                        if let Some(last) = insts.get(start_idx..).and_then(|s| s.last()) {
                            Ok(match last {
                                IrInstruction::Literal { dest, .. } => dest.clone(),
                                IrInstruction::Add { dest, .. } => dest.clone(),
                                IrInstruction::Sub { dest, .. } => dest.clone(),
                                IrInstruction::Mul { dest, .. } => dest.clone(),
                                IrInstruction::Div { dest, .. } => dest.clone(),
                                IrInstruction::Mod { dest, .. } => dest.clone(),
                                IrInstruction::Eq { dest, .. } => dest.clone(),
                                IrInstruction::Ne { dest, .. } => dest.clone(),
                                IrInstruction::Lt { dest, .. } => dest.clone(),
                                IrInstruction::Le { dest, .. } => dest.clone(),
                                IrInstruction::Gt { dest, .. } => dest.clone(),
                                IrInstruction::Ge { dest, .. } => dest.clone(),
                                IrInstruction::And { dest, .. } => dest.clone(),
                                IrInstruction::Or { dest, .. } => dest.clone(),
                                IrInstruction::Not { dest, .. } => dest.clone(),
                                IrInstruction::BitNot { dest, .. } => dest.clone(),
                                IrInstruction::BitAnd { dest, .. } => dest.clone(),
                                IrInstruction::BitOr { dest, .. } => dest.clone(),
                                IrInstruction::BitXor { dest, .. } => dest.clone(),
                                IrInstruction::Shl { dest, .. } => dest.clone(),
                                IrInstruction::Shr { dest, .. } => dest.clone(),
                                IrInstruction::Neg { dest, .. } => dest.clone(),
                                IrInstruction::Call { dest, .. } |
                                IrInstruction::ClosureCall { dest, .. } => dest.clone().unwrap_or_else(|| ctx.next_temp()),
                                IrInstruction::Move { dest, .. } => dest.clone(),
                                IrInstruction::Index { dest, .. } => dest.clone(),
                                IrInstruction::FieldAccess { dest, .. } => dest.clone(),
                                IrInstruction::BorrowImm { dest, .. } => dest.clone(),
                                IrInstruction::BorrowMut { dest, .. } => dest.clone(),
                                IrInstruction::VolatileLoad { dest, .. } => dest.clone(),
                                IrInstruction::EnumInit { dest, .. } => dest.clone(),
                                IrInstruction::EnumTag { dest, .. } => dest.clone(),
                                IrInstruction::EnumPayload { dest, .. } => dest.clone(),
                                _ => ctx.next_temp(),
                            })
                        } else {
                            Ok(ctx.next_temp())
                        }
                    }
                }
            }
            
            // Evaluate left and right operands
            let left_val = get_expr_value(left, &mut insts, ctx)?;
            let right_val = get_expr_value(right, &mut insts, ctx)?;
            
            let dest = ctx.next_temp();
            let ir_op = match op {
                crate::ast::BinaryOperator::Add => IrInstruction::Add {
                    left: left_val,
                    right: right_val,
                    dest: dest.clone(),
                },
                crate::ast::BinaryOperator::Sub => IrInstruction::Sub {
                    left: left_val,
                    right: right_val,
                    dest: dest.clone(),
                },
                crate::ast::BinaryOperator::Mul => IrInstruction::Mul {
                    left: left_val,
                    right: right_val,
                    dest: dest.clone(),
                },
                crate::ast::BinaryOperator::Div => IrInstruction::Div {
                    left: left_val,
                    right: right_val,
                    dest: dest.clone(),
                },
                crate::ast::BinaryOperator::Mod => IrInstruction::Mod {
                    left: left_val,
                    right: right_val,
                    dest: dest.clone(),
                },
                crate::ast::BinaryOperator::Eq => IrInstruction::Eq {
                    left: left_val,
                    right: right_val,
                    dest: dest.clone(),
                },
                crate::ast::BinaryOperator::Ne => IrInstruction::Ne {
                    left: left_val,
                    right: right_val,
                    dest: dest.clone(),
                },
                crate::ast::BinaryOperator::Lt => IrInstruction::Lt {
                    left: left_val,
                    right: right_val,
                    dest: dest.clone(),
                },
                crate::ast::BinaryOperator::Le => IrInstruction::Le {
                    left: left_val,
                    right: right_val,
                    dest: dest.clone(),
                },
                crate::ast::BinaryOperator::Gt => IrInstruction::Gt {
                    left: left_val,
                    right: right_val,
                    dest: dest.clone(),
                },
                crate::ast::BinaryOperator::Ge => IrInstruction::Ge {
                    left: left_val,
                    right: right_val,
                    dest: dest.clone(),
                },
                crate::ast::BinaryOperator::And => IrInstruction::And {
                    left: left_val,
                    right: right_val,
                    dest: dest.clone(),
                },
                crate::ast::BinaryOperator::Or => IrInstruction::Or {
                    left: left_val,
                    right: right_val,
                    dest: dest.clone(),
                },
                crate::ast::BinaryOperator::BitAnd => IrInstruction::BitAnd {
                    left: left_val,
                    right: right_val,
                    dest: dest.clone(),
                },
                crate::ast::BinaryOperator::BitOr => IrInstruction::BitOr {
                    left: left_val,
                    right: right_val,
                    dest: dest.clone(),
                },
                crate::ast::BinaryOperator::BitXor => IrInstruction::BitXor {
                    left: left_val,
                    right: right_val,
                    dest: dest.clone(),
                },
                crate::ast::BinaryOperator::Shl => IrInstruction::Shl {
                    left: left_val,
                    right: right_val,
                    dest: dest.clone(),
                },
                crate::ast::BinaryOperator::Shr => IrInstruction::Shr {
                    left: left_val,
                    right: right_val,
                    dest: dest.clone(),
                },
            };
            insts.push(ir_op);
            Ok(insts)
        }
        crate::ast::Expression::UnaryOp { op, operand } => {
            let mut insts = Vec::new();
            
            // Get operand value — handle Variable directly (ast_expr_to_ir returns empty Vec for variables)
            let operand_temp = if let crate::ast::Expression::Variable(name) = operand.as_ref() {
                // Check const values first
                if let Some(lit) = ctx.const_values.get(name).cloned() {
                    let dest = ctx.next_temp();
                    insts.push(IrInstruction::Literal {
                        value: lit,
                        dest: dest.clone(),
                    });
                    dest
                } else {
                    ctx.get_variable(name)
                        .ok_or_else(|| format!("Undefined variable: {}", name))?
                }
            } else {
                let expr_insts = ast_expr_to_ir(operand, ctx)?;
                if let Some(last) = expr_insts.last() {
                    let val = match last {
                        IrInstruction::Literal { dest, .. } => dest.clone(),
                        IrInstruction::Add { dest, .. } => dest.clone(),
                        IrInstruction::Sub { dest, .. } => dest.clone(),
                        IrInstruction::Mul { dest, .. } => dest.clone(),
                        IrInstruction::Div { dest, .. } => dest.clone(),
                        IrInstruction::Mod { dest, .. } => dest.clone(),
                        IrInstruction::BitAnd { dest, .. } => dest.clone(),
                        IrInstruction::BitOr { dest, .. } => dest.clone(),
                        IrInstruction::BitXor { dest, .. } => dest.clone(),
                        IrInstruction::BitNot { dest, .. } => dest.clone(),
                        IrInstruction::Shl { dest, .. } => dest.clone(),
                        IrInstruction::Shr { dest, .. } => dest.clone(),
                        IrInstruction::Neg { dest, .. } => dest.clone(),
                        IrInstruction::Not { dest, .. } => dest.clone(),
                        IrInstruction::Move { dest, .. } => dest.clone(),
                        IrInstruction::Index { dest, .. } => dest.clone(),
                        IrInstruction::FieldAccess { dest, .. } => dest.clone(),
                        IrInstruction::Call { dest, .. } |
                        IrInstruction::ClosureCall { dest, .. } => dest.clone().unwrap_or_else(|| ctx.next_temp()),
                        _ => ctx.next_temp(),
                    };
                    insts.extend(expr_insts);
                    val
                } else {
                    ctx.next_temp()
                }
            };
            
            let dest = ctx.next_temp();
            match op {
                crate::ast::UnaryOperator::Neg => {
                    insts.push(IrInstruction::Neg {
                        operand: operand_temp,
                        dest,
                    });
                }
                crate::ast::UnaryOperator::Not => {
                    insts.push(IrInstruction::Not {
                        operand: operand_temp,
                        dest,
                    });
                }
                crate::ast::UnaryOperator::BitNot => {
                    insts.push(IrInstruction::BitNot {
                        operand: operand_temp,
                        dest,
                    });
                }
                crate::ast::UnaryOperator::Ref => {
                    insts.push(IrInstruction::AddrOf {
                        operand: operand_temp,
                        dest,
                    });
                }
                crate::ast::UnaryOperator::Deref => {
                    insts.push(IrInstruction::PtrDeref {
                        operand: operand_temp,
                        dest,
                    });
                }
            }
            Ok(insts)
        }
        crate::ast::Expression::Call { callee, args, type_args: _ } => {
            let mut insts = Vec::new();
            let mut arg_values = Vec::new();
            
            // Evaluate arguments - handle variables specially
            for arg in args {
                let arg_value = match arg {
                    crate::ast::Expression::Variable(name) => {
                        // Variable reference - get value directly
                        ctx.get_variable(name)
                            .ok_or_else(|| format!("Undefined variable: {}", name))?
                    }
                    _ => {
                        // Evaluate expression and get result
                        let arg_insts = ast_expr_to_ir(arg, ctx)?;
                        let start_idx = insts.len();
                        insts.extend(arg_insts);
                        
                        if let Some(last) = insts.get(start_idx..).and_then(|s| s.last()) {
                            match last {
                                IrInstruction::Literal { dest, .. } => dest.clone(),
                                IrInstruction::Add { dest, .. } => dest.clone(),
                                IrInstruction::Sub { dest, .. } => dest.clone(),
                                IrInstruction::Mul { dest, .. } => dest.clone(),
                                IrInstruction::Div { dest, .. } => dest.clone(),
                                IrInstruction::Mod { dest, .. } => dest.clone(),
                                IrInstruction::BitAnd { dest, .. } => dest.clone(),
                                IrInstruction::BitOr { dest, .. } => dest.clone(),
                                IrInstruction::BitXor { dest, .. } => dest.clone(),
                                IrInstruction::BitNot { dest, .. } => dest.clone(),
                                IrInstruction::Shl { dest, .. } => dest.clone(),
                                IrInstruction::Shr { dest, .. } => dest.clone(),
                                IrInstruction::Neg { dest, .. } => dest.clone(),
                                IrInstruction::Not { dest, .. } => dest.clone(),
                                IrInstruction::Eq { dest, .. } => dest.clone(),
                                IrInstruction::Ne { dest, .. } => dest.clone(),
                                IrInstruction::Lt { dest, .. } => dest.clone(),
                                IrInstruction::Le { dest, .. } => dest.clone(),
                                IrInstruction::Gt { dest, .. } => dest.clone(),
                                IrInstruction::Ge { dest, .. } => dest.clone(),
                                IrInstruction::And { dest, .. } => dest.clone(),
                                IrInstruction::Or { dest, .. } => dest.clone(),
                                IrInstruction::Call { dest, .. } |
                                IrInstruction::ClosureCall { dest, .. } => dest.clone().unwrap_or_else(|| ctx.next_temp()),
                                IrInstruction::Move { dest, .. } => dest.clone(),
                                IrInstruction::VolatileLoad { dest, .. } => dest.clone(),
                                IrInstruction::FieldAccess { dest, .. } => dest.clone(),
                                IrInstruction::StructInit { dest, .. } => dest.clone(),
                                IrInstruction::Index { dest, .. } => dest.clone(),
                                IrInstruction::ArrayInit { dest, .. } => dest.clone(),
                                _ => ctx.next_temp(),
                            }
                        } else {
                            ctx.next_temp()
                        }
                    }
                };
                arg_values.push(arg_value);
            }
            
            // Get callee name
            let func_name = match callee.as_ref() {
                crate::ast::Expression::Variable(name) => name.clone(),
                _ => return Err("Function call must be to a variable/identifier".to_string()),
            };
            
            // Check for builtin functions
            let dest = ctx.next_temp();
            
            if func_name == "tag" {
                // tag(enum_value) -> extract discriminant as i32
                if arg_values.len() != 1 {
                    return Err("tag() takes exactly 1 argument".to_string());
                }
                insts.push(IrInstruction::EnumTag {
                    value: arg_values.remove(0),
                    dest: dest,
                });
            } else if func_name == "payload" {
                // payload(enum_value) -> extract payload (assumes correct variant)
                if arg_values.len() != 1 {
                    return Err("payload() takes exactly 1 argument".to_string());
                }
                insts.push(IrInstruction::EnumPayload {
                    value: arg_values.remove(0),
                    variant: String::new(), // Generic payload extraction
                    dest: dest,
                });
            } else if func_name == "read_volatile" {
                if arg_values.len() != 1 {
                    return Err("read_volatile() takes exactly 1 argument".to_string());
                }
                insts.push(IrInstruction::VolatileLoad {
                    addr: arg_values.remove(0),
                    dest,
                });
            } else if func_name == "write_volatile" {
                if arg_values.len() != 2 {
                    return Err("write_volatile() takes exactly 2 arguments".to_string());
                }
                let addr = arg_values.remove(0);
                let value = arg_values.remove(0);
                insts.push(IrInstruction::VolatileStore { addr, value });
            } else if ctx.is_closure_variable(&func_name) {
                // Call through a closure value binding
                insts.push(IrInstruction::ClosureCall {
                    closure: IrValue::Variable(func_name),
                    args: arg_values,
                    dest: Some(dest),
                });
            } else {
                // Regular function call
                insts.push(IrInstruction::Call {
                    func: func_name,
                    args: arg_values,
                    dest: Some(dest),
                });
            }
            Ok(insts)
        }
        crate::ast::Expression::Borrow { mutable, expr } => {
            let mut insts = ast_expr_to_ir(expr, ctx)?;
            let src_temp = if let Some(last) = insts.last() {
                match last {
                    IrInstruction::Literal { dest, .. } => dest.clone(),
                    IrInstruction::Add { dest, .. } => dest.clone(),
                    IrInstruction::Sub { dest, .. } => dest.clone(),
                    IrInstruction::Mul { dest, .. } => dest.clone(),
                    IrInstruction::Div { dest, .. } => dest.clone(),
                    IrInstruction::BitAnd { dest, .. } => dest.clone(),
                    IrInstruction::BitOr { dest, .. } => dest.clone(),
                    IrInstruction::BitXor { dest, .. } => dest.clone(),
                    IrInstruction::BitNot { dest, .. } => dest.clone(),
                    IrInstruction::Shl { dest, .. } => dest.clone(),
                    IrInstruction::Shr { dest, .. } => dest.clone(),
                    IrInstruction::Move { dest, .. } => dest.clone(),
                    _ => ctx.next_temp(),
                }
            } else {
                ctx.next_temp()
            };
            
            let dest = ctx.next_temp();
            let lifetime = format!("lifetime_{}", ctx.temp_counter);
            
            if *mutable {
                insts.push(IrInstruction::BorrowMut {
                    src: src_temp,
                    dest,
                    lifetime,
                });
            } else {
                insts.push(IrInstruction::BorrowImm {
                    src: src_temp,
                    dest,
                    lifetime,
                });
            }
            Ok(insts)
        }
        crate::ast::Expression::Index { receiver, index } => {
            let mut insts = Vec::new();
            
            // Get the base value - handle Variable receivers specially
            let base_val = match receiver.as_ref() {
                crate::ast::Expression::Variable(name) => {
                    ctx.get_variable(name)
                        .ok_or_else(|| format!("Undefined variable: {}", name))?
                }
                _ => {
                    let receiver_insts = ast_expr_to_ir(receiver, ctx)?;
                    let start_idx = insts.len();
                    insts.extend(receiver_insts);
                    
                    if let Some(last) = insts.get(start_idx..).and_then(|s| s.last()) {
                        match last {
                            IrInstruction::Literal { dest, .. } => dest.clone(),
                            IrInstruction::ArrayInit { dest, .. } => dest.clone(),
                            IrInstruction::Add { dest, .. } => dest.clone(),
                            IrInstruction::Sub { dest, .. } => dest.clone(),
                            IrInstruction::Mul { dest, .. } => dest.clone(),
                            IrInstruction::Div { dest, .. } => dest.clone(),
                            IrInstruction::BitAnd { dest, .. } => dest.clone(),
                            IrInstruction::BitOr { dest, .. } => dest.clone(),
                            IrInstruction::BitXor { dest, .. } => dest.clone(),
                            IrInstruction::BitNot { dest, .. } => dest.clone(),
                            IrInstruction::Shl { dest, .. } => dest.clone(),
                            IrInstruction::Shr { dest, .. } => dest.clone(),
                            IrInstruction::Move { dest, .. } => dest.clone(),
                            _ => ctx.next_temp(),
                        }
                    } else {
                        ctx.next_temp()
                    }
                }
            };
            
            // Get the index value - handle literals as constants
            let index_val = match index.as_ref() {
                crate::ast::Expression::Literal(crate::ast::Literal::Int(n)) => {
                    IrValue::Constant(IrLiteral::Int(*n))
                }
                crate::ast::Expression::Variable(name) => {
                    ctx.get_variable(name)
                        .ok_or_else(|| format!("Undefined variable: {}", name))?
                }
                _ => {
                    let index_insts = ast_expr_to_ir(index, ctx)?;
                    let start_idx = insts.len();
                    insts.extend(index_insts);
                    
                    if let Some(last) = insts.get(start_idx..).and_then(|s| s.last()) {
                        match last {
                            IrInstruction::Literal { dest, .. } => dest.clone(),
                            IrInstruction::Add { dest, .. } => dest.clone(),
                            _ => ctx.next_temp(),
                        }
                    } else {
                        ctx.next_temp()
                    }
                }
            };
            
            let dest = ctx.next_temp();
            insts.push(IrInstruction::Index {
                base: base_val,
                index: index_val,
                dest,
            });
            Ok(insts)
        }
        crate::ast::Expression::FieldAccess { receiver, field } => {
            let mut insts = Vec::new();
            
            // Get the base value - handle Variable receivers specially
            let base_val = match receiver.as_ref() {
                crate::ast::Expression::Variable(name) => {
                    ctx.get_variable(name)
                        .ok_or_else(|| format!("Undefined variable: {}", name))?
                }
                _ => {
                    let receiver_insts = ast_expr_to_ir(receiver, ctx)?;
                    let val = if let Some(last) = receiver_insts.last() {
                        match last {
                            IrInstruction::Literal { dest, .. } => dest.clone(),
                            IrInstruction::Add { dest, .. } => dest.clone(),
                            IrInstruction::Sub { dest, .. } => dest.clone(),
                            IrInstruction::Mul { dest, .. } => dest.clone(),
                            IrInstruction::Div { dest, .. } => dest.clone(),
                            IrInstruction::BitAnd { dest, .. } => dest.clone(),
                            IrInstruction::BitOr { dest, .. } => dest.clone(),
                            IrInstruction::BitXor { dest, .. } => dest.clone(),
                            IrInstruction::BitNot { dest, .. } => dest.clone(),
                            IrInstruction::Shl { dest, .. } => dest.clone(),
                            IrInstruction::Shr { dest, .. } => dest.clone(),
                            IrInstruction::Move { dest, .. } => dest.clone(),
                            IrInstruction::StructInit { dest, .. } => dest.clone(),
                            _ => ctx.next_temp(),
                        }
                    } else {
                        ctx.next_temp()
                    };
                    insts.extend(receiver_insts);
                    val
                }
            };
            
            let dest = ctx.next_temp();
            insts.push(IrInstruction::FieldAccess {
                base: base_val,
                field: field.clone(),
                dest,
            });
            Ok(insts)
        }
        crate::ast::Expression::Block(block) => {
            let mut insts = Vec::new();
            for stmt in &block.statements {
                insts.extend(ast_stmt_to_ir(stmt, ctx)?);
            }
            Ok(insts)
        }
        crate::ast::Expression::Range { start, end } => {
            let mut insts = Vec::new();
            
            // Handle start (default to 0 if None)
            let start_val = if let Some(s) = start {
                 let s_insts = ast_expr_to_ir(s, ctx)?;
                 let s_val = if let Some(last) = s_insts.last() {
                    match last {
                        IrInstruction::Literal { dest, .. } => dest.clone(),
                         _ => ctx.next_temp(),
                    }
                 } else {
                     ctx.next_temp()
                 };
                 insts.extend(s_insts);
                 s_val
            } else {
                let zero = ctx.next_temp();
                insts.push(IrInstruction::Literal { 
                    value: IrLiteral::Int(0), 
                    dest: zero.clone() 
                });
                zero
            };
            
            // Handle end (error if None for now)
            let end_val = if let Some(e) = end {
                 let e_insts = ast_expr_to_ir(e, ctx)?;
                 let e_val = if let Some(last) = e_insts.last() {
                    match last {
                        IrInstruction::Literal { dest, .. } => dest.clone(),
                         _ => ctx.next_temp(),
                    }
                 } else {
                     ctx.next_temp()
                 };
                 insts.extend(e_insts);
                 e_val
            } else {
                return Err("Range expression must have an end value".to_string());
            };
            
            let dest = ctx.next_temp();
            insts.push(IrInstruction::Range {
                start: start_val,
                end: end_val,
                dest,
            });
            Ok(insts)
        }
        crate::ast::Expression::StructLiteral { ty, fields } => {
            let mut insts = Vec::new();
            
            // Get struct name from type
            let struct_name = match ty {
                crate::ast::Type::Named(name) => name.clone(),
                _ => return Err("Struct literal must have a named type".to_string()),
            };
            
            // Helper to get expression value (duplicated from ast_stmt_to_ir)
            fn get_expr_val(expr: &crate::ast::Expression, insts: &mut Vec<IrInstruction>, ctx: &mut IrContext) -> Result<IrValue, String> {
                match expr {
                    crate::ast::Expression::Variable(name) => {
                        ctx.get_variable(name)
                            .ok_or_else(|| format!("Undefined variable: {}", name))
                    }
                    _ => {
                        let expr_insts = ast_expr_to_ir(expr, ctx)?;
                        let start_idx = insts.len();
                        insts.extend(expr_insts);
                        
                        if let Some(last) = insts.get(start_idx..).and_then(|s| s.last()) {
                            Ok(match last {
                                IrInstruction::Literal { dest, .. } => dest.clone(),
                                IrInstruction::Add { dest, .. } => dest.clone(),
                                IrInstruction::Sub { dest, .. } => dest.clone(),
                                IrInstruction::Mul { dest, .. } => dest.clone(),
                                IrInstruction::Div { dest, .. } => dest.clone(),
                                IrInstruction::Mod { dest, .. } => dest.clone(),
                                IrInstruction::BitAnd { dest, .. } => dest.clone(),
                                IrInstruction::BitOr { dest, .. } => dest.clone(),
                                IrInstruction::BitXor { dest, .. } => dest.clone(),
                                IrInstruction::BitNot { dest, .. } => dest.clone(),
                                IrInstruction::Shl { dest, .. } => dest.clone(),
                                IrInstruction::Shr { dest, .. } => dest.clone(),
                                IrInstruction::Neg { dest, .. } => dest.clone(),
                                IrInstruction::Not { dest, .. } => dest.clone(),
                                IrInstruction::Call { dest, .. } |
                                IrInstruction::ClosureCall { dest, .. } => dest.clone().unwrap_or_else(|| ctx.next_temp()),
                                IrInstruction::Move { dest, .. } => dest.clone(),
                                _ => ctx.next_temp(),
                            })
                        } else {
                            Ok(ctx.next_temp())
                        }
                    }
                }
            }
            
            // Collect field values
            let mut field_values = Vec::new();
            for field_init in fields {
                let val = get_expr_val(&field_init.value, &mut insts, ctx)?;
                field_values.push((field_init.name.clone(), val));
            }
            
            // Emit StructInit instruction
            let dest = ctx.next_temp();
            insts.push(IrInstruction::StructInit {
                struct_name,
                fields: field_values,
                dest: dest.clone(),
            });
            
            Ok(insts)
        }
        // NOTE: Expression::Assignment has been removed from AST
        // Assignments are now Statement::Assign which is handled in ast_stmt_to_ir()
        crate::ast::Expression::Array(elements) => {
            let mut insts = Vec::new();
            
            // Helper to get expression value - use Constant for literals, no separate instructions
            fn get_elem_val(expr: &crate::ast::Expression, insts: &mut Vec<IrInstruction>, ctx: &mut IrContext) -> Result<IrValue, String> {
                match expr {
                    crate::ast::Expression::Variable(name) => {
                        ctx.get_variable(name)
                            .ok_or_else(|| format!("Undefined variable: {}", name))
                    }
                    crate::ast::Expression::Literal(lit) => {
                        // Return as Constant directly - no need for separate Literal instruction
                        let ir_lit = match lit {
                            crate::ast::Literal::Int(n) => IrLiteral::Int(*n),
                            crate::ast::Literal::Float(f) => IrLiteral::Float(*f),
                            crate::ast::Literal::Bool(b) => IrLiteral::Bool(*b),
                            crate::ast::Literal::String(s) => IrLiteral::String(s.clone()),
                            crate::ast::Literal::Char(c) => IrLiteral::Char(*c),
                        };
                        Ok(IrValue::Constant(ir_lit))
                    }
                    _ => {
                        let expr_insts = ast_expr_to_ir(expr, ctx)?;
                        let start_idx = insts.len();
                        insts.extend(expr_insts);
                        
                        if let Some(last) = insts.get(start_idx..).and_then(|s| s.last()) {
                            Ok(match last {
                                IrInstruction::Literal { dest, .. } => dest.clone(),
                                IrInstruction::Add { dest, .. } => dest.clone(),
                                IrInstruction::Sub { dest, .. } => dest.clone(),
                                IrInstruction::Mul { dest, .. } => dest.clone(),
                                IrInstruction::Div { dest, .. } => dest.clone(),
                                _ => ctx.next_temp(),
                            })
                        } else {
                            Ok(ctx.next_temp())
                        }
                    }
                }
            }
            
            // Evaluate all elements
            let mut values = Vec::new();
            for elem in elements {
                let val = get_elem_val(elem, &mut insts, ctx)?;
                values.push(val);
            }
            
            let dest = ctx.next_temp();
            insts.push(IrInstruction::ArrayInit { elements: values, dest });
            Ok(insts)
        }
        crate::ast::Expression::EnumVariant { enum_name, variant, data } => {
            let mut insts = Vec::new();
            
            // Helper to get expression value
            fn get_expr_val(expr: &crate::ast::Expression, insts: &mut Vec<IrInstruction>, ctx: &mut IrContext) -> Result<IrValue, String> {
                match expr {
                    crate::ast::Expression::Variable(name) => {
                        ctx.get_variable(name)
                            .ok_or_else(|| format!("Undefined variable: {}", name))
                    }
                    crate::ast::Expression::Literal(lit) => {
                        let ir_lit = match lit {
                            crate::ast::Literal::Int(n) => IrLiteral::Int(*n),
                            crate::ast::Literal::Float(f) => IrLiteral::Float(*f),
                            crate::ast::Literal::Bool(b) => IrLiteral::Bool(*b),
                            crate::ast::Literal::String(s) => IrLiteral::String(s.clone()),
                            crate::ast::Literal::Char(c) => IrLiteral::Char(*c),
                        };
                        Ok(IrValue::Constant(ir_lit))
                    }
                    _ => {
                        let expr_insts = ast_expr_to_ir(expr, ctx)?;
                        let start_idx = insts.len();
                        insts.extend(expr_insts);
                        
                        if let Some(last) = insts.get(start_idx..).and_then(|s| s.last()) {
                            Ok(match last {
                                IrInstruction::Literal { dest, .. } => dest.clone(),
                                IrInstruction::Add { dest, .. } => dest.clone(),
                                IrInstruction::Sub { dest, .. } => dest.clone(),
                                IrInstruction::Mul { dest, .. } => dest.clone(),
                                IrInstruction::Div { dest, .. } => dest.clone(),
                                IrInstruction::Neg { dest, .. } => dest.clone(),
                                IrInstruction::Not { dest, .. } => dest.clone(),
                                IrInstruction::BitNot { dest, .. } => dest.clone(),
                                IrInstruction::BitAnd { dest, .. } => dest.clone(),
                                IrInstruction::BitOr { dest, .. } => dest.clone(),
                                IrInstruction::BitXor { dest, .. } => dest.clone(),
                                IrInstruction::Shl { dest, .. } => dest.clone(),
                                IrInstruction::Shr { dest, .. } => dest.clone(),
                                IrInstruction::Call { dest, .. } |
                                IrInstruction::ClosureCall { dest, .. } => dest.clone().unwrap_or_else(|| ctx.next_temp()),
                                IrInstruction::Move { dest, .. } => dest.clone(),
                                IrInstruction::EnumInit { dest, .. } => dest.clone(),
                                _ => ctx.next_temp(),
                            })
                        } else {
                            Ok(ctx.next_temp())
                        }
                    }
                }
            }
            
            // Get payload value if present
            let payload = if let Some(data_expr) = data {
                Some(get_expr_val(data_expr, &mut insts, ctx)?)
            } else {
                None
            };
            
            let dest = ctx.next_temp();
            insts.push(IrInstruction::EnumInit {
                enum_name: enum_name.clone(),
                variant: variant.clone(),
                payload,
                dest,
            });
            Ok(insts)
        }
        crate::ast::Expression::Closure { params, return_type: _, body } => {
            // Generate a unique closure ID
            static CLOSURE_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let closure_id = CLOSURE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            
            // Create IR parameters from AST params
            let ir_params: Vec<IrParameter> = params.iter()
                .map(|p| IrParameter {
                    name: p.name.clone(),
                    ty: ast_type_to_ir(&p.ty),
                })
                .collect();
            
            // Add closure parameters to context before evaluating body
            for param in &ir_params {
                let param_value = IrValue::Variable(param.name.clone());
                ctx.add_variable(param.name.clone(), param_value);
            }
            
            // Generate body instructions
            let mut body_insts = ast_expr_to_ir(body, ctx)?;
            
            // Handle identity closures: if body is empty (just a variable reference),
            // generate an explicit Move instruction to capture the return value
            if body_insts.is_empty() {
                if let crate::ast::Expression::Variable(var_name) = body.as_ref() {
                    let src = IrValue::Variable(var_name.clone());
                    let ret_dest = ctx.next_temp();
                    body_insts.push(IrInstruction::Move {
                        src,
                        dest: ret_dest,
                    });
                }
            }
            
            // Detect captured variables (variables used in body but not in params)
            let param_names: std::collections::HashSet<String> = params.iter()
                .map(|p| p.name.clone())
                .collect();
            let mut captures: Vec<String> = Vec::new();
            collect_free_variables(body, &param_names, &mut captures);
            if !captures.is_empty() {
                return Err(format!(
                    "Closure captures variable(s) {} — closures with captures are not yet supported. \
                     Use function parameters instead.",
                    captures.iter().map(|c| format!("'{}'", c)).collect::<Vec<_>>().join(", ")
                ));
            }
            
            // Create result destination
            let dest = ctx.next_temp();
            
            // Emit ClosureCreate instruction
            let closure_create = IrInstruction::ClosureCreate {
                closure_id,
                params: ir_params,
                body: body_insts,
                captures,
                dest,
            };

            // The closure expression result itself is a closure value
            if let IrInstruction::ClosureCreate { dest, .. } = &closure_create {
                ctx.set_value_is_closure(dest, true);
            }

            Ok(vec![closure_create])
        }
        crate::ast::Expression::StaticCall { type_name, method, args } => {
            // Static method call: Point::sum(10, 20) -> Call to Point_sum(10, 20)
            let mut insts = Vec::new();
            let mut arg_values = Vec::new();
            
            // Evaluate arguments
            for arg in args {
                let arg_value = match arg {
                    crate::ast::Expression::Variable(name) => {
                        ctx.get_variable(name)
                            .ok_or_else(|| format!("Undefined variable: {}", name))?
                    }
                    _ => {
                        let arg_insts = ast_expr_to_ir(arg, ctx)?;
                        let start_idx = insts.len();
                        insts.extend(arg_insts);
                        
                        if let Some(last) = insts.get(start_idx..).and_then(|s| s.last()) {
                            match last {
                                IrInstruction::Literal { dest, .. } => dest.clone(),
                                IrInstruction::Add { dest, .. } => dest.clone(),
                                IrInstruction::Sub { dest, .. } => dest.clone(),
                                IrInstruction::Mul { dest, .. } => dest.clone(),
                                IrInstruction::Div { dest, .. } => dest.clone(),
                                IrInstruction::Call { dest, .. } |
                                IrInstruction::ClosureCall { dest, .. } => dest.clone().unwrap_or_else(|| ctx.next_temp()),
                                IrInstruction::Move { dest, .. } => dest.clone(),
                                _ => ctx.next_temp(),
                            }
                        } else {
                            ctx.next_temp()
                        }
                    }
                };
                arg_values.push(arg_value);
            }

            // Reinterpret `EnumName::Variant(...)` as enum construction once enum metadata
            // is available. This resolves parser ambiguity for single-arg constructor syntax.
            if ctx.has_enum(type_name) && ctx.enum_has_variant(type_name, method) {
                let payload = match arg_values.len() {
                    0 => None,
                    1 => Some(arg_values.remove(0)),
                    count => {
                        return Err(format!(
                            "Enum variant '{}::{}' expects at most one payload value, got {}",
                            type_name, method, count
                        ))
                    }
                };

                let dest = ctx.next_temp();
                insts.push(IrInstruction::EnumInit {
                    enum_name: type_name.clone(),
                    variant: method.clone(),
                    payload,
                    dest,
                });
                return Ok(insts);
            }
            
            // Mangle name: TypeName_methodName
            let mangled_name = format!("{}_{}", type_name, method);
            let dest = ctx.next_temp();
            
            insts.push(IrInstruction::Call {
                func: mangled_name,
                args: arg_values,
                dest: Some(dest),
            });
            Ok(insts)
        }
        crate::ast::Expression::MethodCall { receiver, method, args } => {
            // Instance method call: p.distance() -> Call to TypeName_distance(&p, ...)
            let mut insts = Vec::new();
            let mut arg_values = Vec::new();
            
            // First, get the receiver value (becomes first argument as &self)
            let receiver_val = match receiver.as_ref() {
                crate::ast::Expression::Variable(name) => {
                    ctx.get_variable(name)
                        .ok_or_else(|| format!("Undefined variable: {}", name))?
                }
                _ => {
                    let recv_insts = ast_expr_to_ir(receiver, ctx)?;
                    let start_idx = insts.len();
                    insts.extend(recv_insts);
                    
                    if let Some(last) = insts.get(start_idx..).and_then(|s| s.last()) {
                        match last {
                            IrInstruction::StructInit { dest, .. } => dest.clone(),
                            IrInstruction::Move { dest, .. } => dest.clone(),
                            _ => ctx.next_temp(),
                        }
                    } else {
                        ctx.next_temp()
                    }
                }
            };
            arg_values.push(receiver_val);
            
            // Evaluate remaining arguments
            for arg in args {
                let arg_value = match arg {
                    crate::ast::Expression::Variable(name) => {
                        ctx.get_variable(name)
                            .ok_or_else(|| format!("Undefined variable: {}", name))?
                    }
                    _ => {
                        let arg_insts = ast_expr_to_ir(arg, ctx)?;
                        let start_idx = insts.len();
                        insts.extend(arg_insts);
                        
                        if let Some(last) = insts.get(start_idx..).and_then(|s| s.last()) {
                            match last {
                                IrInstruction::Literal { dest, .. } => dest.clone(),
                                IrInstruction::Add { dest, .. } => dest.clone(),
                                IrInstruction::Sub { dest, .. } => dest.clone(),
                                IrInstruction::Mul { dest, .. } => dest.clone(),
                                IrInstruction::Div { dest, .. } => dest.clone(),
                                IrInstruction::Call { dest, .. } |
                                IrInstruction::ClosureCall { dest, .. } => dest.clone().unwrap_or_else(|| ctx.next_temp()),
                                IrInstruction::Move { dest, .. } => dest.clone(),
                                _ => ctx.next_temp(),
                            }
                        } else {
                            ctx.next_temp()
                        }
                    }
                };
                arg_values.push(arg_value);
            }
            
            
            // Get type name from receiver variable for method name mangling
            let type_name = match receiver.as_ref() {
                crate::ast::Expression::Variable(name) => {
                    ctx.get_variable_type(name)
                        .cloned()
                        .unwrap_or_else(|| "UNKNOWN_TYPE".to_string())
                }
                _ => "UNKNOWN_TYPE".to_string(), // Complex expressions - would need full type inference
            };
            let mangled_name = format!("{}_{}", type_name, method);
            let dest = ctx.next_temp();
            
            insts.push(IrInstruction::Call {
                func: mangled_name,
                args: arg_values,
                dest: Some(dest),
            });
            Ok(insts)
        }
        crate::ast::Expression::Try(inner) => {
            // Desugar `expr?` into:
            //   let result = <inner>;
            //   let tag = EnumTag(result);
            //   if tag == ok_tag { unwrapped = EnumPayload(result, "Ok") }
            //   else { return result; }    // propagate Err/None as-is
            let mut insts = Vec::new();

            // Pre-allocate the result variable so it's visible after branches
            let try_result = ctx.next_temp();

            // 1. Evaluate inner expression
            let result_val = match inner.as_ref() {
                crate::ast::Expression::Variable(name) => {
                    ctx.get_variable(name)
                        .ok_or_else(|| format!("Undefined variable: {}", name))?
                }
                _ => {
                    let inner_insts = ast_expr_to_ir(inner, ctx)?;
                    let start_idx = insts.len();
                    insts.extend(inner_insts);
                    if let Some(last) = insts.get(start_idx..).and_then(|s| s.last()) {
                        match last {
                            IrInstruction::Call { dest, .. } |
                            IrInstruction::ClosureCall { dest, .. } => dest.clone().unwrap_or_else(|| ctx.next_temp()),
                            IrInstruction::Move { dest, .. } => dest.clone(),
                            IrInstruction::EnumInit { dest, .. } => dest.clone(),
                            IrInstruction::Literal { dest, .. } => dest.clone(),
                            _ => ctx.next_temp(),
                        }
                    } else {
                        ctx.next_temp()
                    }
                }
            };

            // 2. Extract tag from the result
            let tag_temp = ctx.next_temp();
            insts.push(IrInstruction::EnumTag {
                value: result_val.clone(),
                dest: tag_temp.clone(),
            });

            // 3. Compare with the "Ok" tag (tag 0) — convention: first variant is the success variant
            let ok_tag_val = ctx.next_temp();
            let ok_tag = ctx.get_variant_tag("Ok").unwrap_or(0);
            insts.push(IrInstruction::Literal {
                value: IrLiteral::Int(ok_tag as i64),
                dest: ok_tag_val.clone(),
            });

            let cond_temp = ctx.next_temp();
            insts.push(IrInstruction::Eq {
                left: tag_temp,
                right: ok_tag_val,
                dest: cond_temp.clone(),
            });

            // 4. Branch: if tag == ok_tag goto ok_label, else goto err_label
            let try_ok_label = format!("try_ok_{}", ctx.temp_counter);
            let try_err_label = format!("try_err_{}", ctx.temp_counter);
            let try_end_label = format!("try_end_{}", ctx.temp_counter);

            insts.push(IrInstruction::BranchCond {
                condition: cond_temp,
                true_label: try_ok_label.clone(),
                false_label: try_err_label.clone(),
            });

            // 5. Error path: return the original result value (propagate Err/None)
            insts.push(IrInstruction::Label { name: try_err_label });
            insts.push(IrInstruction::Return {
                value: Some(result_val.clone()),
            });

            // 6. Ok path: extract payload and store to pre-allocated result
            insts.push(IrInstruction::Label { name: try_ok_label });
            let unwrapped = ctx.next_temp();
            insts.push(IrInstruction::EnumPayload {
                value: result_val,
                variant: "Ok".to_string(),
                dest: unwrapped.clone(),
            });
            // Store the unwrapped value into the pre-allocated try_result
            insts.push(IrInstruction::Move {
                src: unwrapped,
                dest: try_result.clone(),
            });
            insts.push(IrInstruction::Branch { label: try_end_label.clone() });

            // 7. Join point — try_result is now the final Move destination
            insts.push(IrInstruction::Label { name: try_end_label });

            // Emit a final Move so get_expr_value sees a Move as the last instruction
            let final_dest = ctx.next_temp();
            insts.push(IrInstruction::Move {
                src: try_result,
                dest: final_dest,
            });

            Ok(insts)
        }
        // Cast expression: lower inner expr then emit Cast instruction with target type
        crate::ast::Expression::Cast { expr, ty } => {
            let expr_insts = ast_expr_to_ir(expr, ctx)?;
            let mut insts = Vec::new();
            
            // Extract result from inner expression (same pattern as UnaryOp)
            let operand_temp = if let Some(last) = expr_insts.last() {
                let val = match last {
                    IrInstruction::Literal { dest, .. } => dest.clone(),
                    IrInstruction::Add { dest, .. } => dest.clone(),
                    IrInstruction::Sub { dest, .. } => dest.clone(),
                    IrInstruction::Mul { dest, .. } => dest.clone(),
                    IrInstruction::Div { dest, .. } => dest.clone(),
                    IrInstruction::Mod { dest, .. } => dest.clone(),
                    IrInstruction::BitAnd { dest, .. } => dest.clone(),
                    IrInstruction::BitOr { dest, .. } => dest.clone(),
                    IrInstruction::BitXor { dest, .. } => dest.clone(),
                    IrInstruction::BitNot { dest, .. } => dest.clone(),
                    IrInstruction::Shl { dest, .. } => dest.clone(),
                    IrInstruction::Shr { dest, .. } => dest.clone(),
                    IrInstruction::Neg { dest, .. } => dest.clone(),
                    IrInstruction::Not { dest, .. } => dest.clone(),
                    IrInstruction::Move { dest, .. } => dest.clone(),
                    IrInstruction::Index { dest, .. } => dest.clone(),
                    IrInstruction::FieldAccess { dest, .. } => dest.clone(),
                    IrInstruction::Cast { dest, .. } => dest.clone(),
                    IrInstruction::Call { dest, .. } |
                    IrInstruction::ClosureCall { dest, .. } => dest.clone().unwrap_or_else(|| ctx.next_temp()),
                    _ => ctx.next_temp(),
                };
                insts.extend(expr_insts);
                val
            } else {
                ctx.next_temp()
            };
            
            let dest = ctx.next_temp();
            let target_ir_type = ast_type_to_ir(ty);
            insts.push(IrInstruction::Cast {
                operand: operand_temp,
                target_type: target_ir_type,
                dest,
            });
            Ok(insts)
        }
        _ => Err(format!(
            "IR lowering not implemented for expression variant '{}'. \
             Use statement form or extend ast_expr_to_ir lowering.",
            expression_kind(expr)
        )),
    }
}

fn ast_type_to_ir(ty: &crate::ast::Type) -> IrType {
    match ty {
        crate::ast::Type::Int(int_ty) => {
            let ir_int_ty = match int_ty {
                crate::ast::IntType::I8 => IntType::I8,
                crate::ast::IntType::I16 => IntType::I16,
                crate::ast::IntType::I32 => IntType::I32,
                crate::ast::IntType::I64 => IntType::I64,
                crate::ast::IntType::I128 => IntType::I128,
                crate::ast::IntType::U8 => IntType::U8,
                crate::ast::IntType::U16 => IntType::U16,
                crate::ast::IntType::U32 => IntType::U32,
                crate::ast::IntType::U64 => IntType::U64,
                crate::ast::IntType::U128 => IntType::U128,
                crate::ast::IntType::ISize => IntType::ISize,
                crate::ast::IntType::USize => IntType::USize,
            };
            IrType::Int(ir_int_ty)
        }
        crate::ast::Type::Float(float_ty) => {
            let ir_float_ty = match float_ty {
                crate::ast::FloatType::F32 => FloatType::F32,
                crate::ast::FloatType::F64 => FloatType::F64,
            };
            IrType::Float(ir_float_ty)
        }
        crate::ast::Type::Bool => IrType::Bool,
        crate::ast::Type::String => IrType::String,
        crate::ast::Type::Str => IrType::Str,
        crate::ast::Type::Unit => IrType::Unit,
        crate::ast::Type::Named(name) => IrType::Named(name.clone()),
        // TypeParam must be resolved by monomorphize_generics() before IR lowering.
        // If one reaches here, monomorphization failed — this is a compiler bug.
        crate::ast::Type::TypeParam(name) => {
            eprintln!("error: unresolved type parameter '{}' reached IR lowering. \
                      Monomorphization should have resolved this. \
                      Provide explicit type arguments at the call site.", name);
            // Return i64 to allow compilation to continue and show downstream errors,
            // but the error message above makes the problem clear.
            return IrType::Int(IntType::I64);
        }
        crate::ast::Type::Pointer { mutable, inner } => IrType::Pointer {
            mutable: *mutable,
            inner: Box::new(ast_type_to_ir(inner)),
        },
        crate::ast::Type::Reference { mutable, lifetime, inner } => IrType::Reference {
            mutable: *mutable,
            lifetime: lifetime.clone(),
            inner: Box::new(ast_type_to_ir(inner)),
        },
        crate::ast::Type::Array { inner, size } => IrType::Array {
            inner: Box::new(ast_type_to_ir(inner)),
            size: *size,
        },
        crate::ast::Type::Tuple(types) => IrType::Tuple(types.iter().map(ast_type_to_ir).collect()),
        crate::ast::Type::Function { params, return_type } => IrType::Function {
            params: params.iter().map(ast_type_to_ir).collect(),
            return_type: Box::new(ast_type_to_ir(return_type)),
        },
        crate::ast::Type::Never => IrType::Unit,  // Never type maps to void/unit at IR level
    }
}

#[cfg(test)]
mod tests {
    use super::{ast_to_ir, IrInstruction};
    use crate::ast::{
        Block, Enum, EnumVariant, Expression, Function, IntType, Item, Literal, Program, Statement,
        Type,
    };

    fn program_with_items_and_expr(mut items: Vec<Item>, expr: Expression) -> Program {
        items.push(Item::Function(Function {
            name: "main".to_string(),
            type_params: vec![],
            params: vec![],
            return_type: None,
            body: Block {
                statements: vec![Statement::Expr(expr)],
            },
            is_unsafe: false,
            is_pub: false,
            profile: None,
        }));

        Program { items }
    }

    fn program_with_expr(expr: Expression) -> Program {
        program_with_items_and_expr(Vec::new(), expr)
    }

    fn main_instructions(module: &super::IrModule) -> &[IrInstruction] {
        let main = module
            .functions
            .iter()
            .find(|f| f.name == "main")
            .expect("expected main function in IR");
        &main.body.instructions
    }

    fn status_enum_item() -> Item {
        Item::Enum(Enum {
            name: "Status".to_string(),
            type_params: vec![],
            variants: vec![
                EnumVariant {
                    name: "Ok".to_string(),
                    data: None,
                },
                EnumVariant {
                    name: "Failed".to_string(),
                    data: Some(Type::Int(IntType::I64)),
                },
            ],
            is_pub: false,
        })
    }

    #[test]
    fn unsupported_expression_returns_diagnostic_instead_of_panicking() {
        let program = program_with_expr(Expression::Unsafe(Box::new(Expression::Literal(
            Literal::Int(1),
        ))));

        let result = ast_to_ir(&program);
        assert!(result.is_err(), "expected lowering error, got {:?}", result);

        let err = result.err().unwrap();
        assert!(
            err.contains("IR lowering not implemented for expression variant 'Unsafe'"),
            "unexpected error message: {}",
            err
        );
    }

    #[test]
    fn enum_constructor_static_call_with_no_args_lowers_to_enum_init() {
        let program = program_with_items_and_expr(
            vec![status_enum_item()],
            Expression::StaticCall {
                type_name: "Status".to_string(),
                method: "Ok".to_string(),
                args: vec![],
            },
        );

        let ir = ast_to_ir(&program).expect("IR lowering must succeed");
        let instructions = main_instructions(&ir);

        assert!(
            instructions.iter().any(|inst| matches!(
                inst,
                IrInstruction::EnumInit {
                    enum_name,
                    variant,
                    payload: None,
                    ..
                } if enum_name == "Status" && variant == "Ok"
            )),
            "expected EnumInit for Status::Ok, got {:?}",
            instructions
        );
        assert!(
            !instructions.iter().any(|inst| matches!(
                inst,
                IrInstruction::Call { func, .. } if func == "Status_Ok"
            )),
            "unexpected mangled call for enum constructor: {:?}",
            instructions
        );
    }

    #[test]
    fn enum_constructor_static_call_with_payload_lowers_to_enum_init() {
        let program = program_with_items_and_expr(
            vec![status_enum_item()],
            Expression::StaticCall {
                type_name: "Status".to_string(),
                method: "Failed".to_string(),
                args: vec![Expression::Literal(Literal::Int(42))],
            },
        );

        let ir = ast_to_ir(&program).expect("IR lowering must succeed");
        let instructions = main_instructions(&ir);

        assert!(
            instructions.iter().any(|inst| matches!(
                inst,
                IrInstruction::EnumInit {
                    enum_name,
                    variant,
                    payload: Some(_),
                    ..
                } if enum_name == "Status" && variant == "Failed"
            )),
            "expected EnumInit with payload for Status::Failed, got {:?}",
            instructions
        );
        assert!(
            !instructions.iter().any(|inst| matches!(
                inst,
                IrInstruction::Call { func, .. } if func == "Status_Failed"
            )),
            "unexpected mangled call for enum constructor: {:?}",
            instructions
        );
    }

    #[test]
    fn non_enum_static_call_stays_mangled_function_call() {
        let program = program_with_expr(Expression::StaticCall {
            type_name: "Point".to_string(),
            method: "sum".to_string(),
            args: vec![Expression::Literal(Literal::Int(1))],
        });

        let ir = ast_to_ir(&program).expect("IR lowering must succeed");
        let instructions = main_instructions(&ir);

        assert!(
            instructions.iter().any(|inst| matches!(
                inst,
                IrInstruction::Call { func, .. } if func == "Point_sum"
            )),
            "expected mangled static call for non-enum type, got {:?}",
            instructions
        );
        assert!(
            !instructions.iter().any(|inst| matches!(
                inst,
                IrInstruction::EnumInit { enum_name, .. } if enum_name == "Point"
            )),
            "unexpected enum constructor lowering for non-enum static call: {:?}",
            instructions
        );
    }
}
