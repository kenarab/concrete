use std::collections::HashMap;

use concrete_session::Session;


use self::errors::LinearityError;
pub mod errors;


use std::path::PathBuf;
//use concrete_ir::{ProgramBody, FnBody};
//use concrete_ast::Program{ ProgramBody, FnBody };
use concrete_ast::Program;
//use concrete_ast::modules::{Module, ModuleDefItem};
use concrete_ast::modules::ModuleDefItem;
//use concrete_ast::functions::FunctionDef;
use concrete_ast::expressions::{Expression, StructInitField, PathOp, ValueExpr};
//use concrete_ast::statements::{Statement, AssignStmt, LetStmt, WhileStmt, ForStmt, LetStmtTarget, Binding};
use concrete_ast::statements::{Statement, AssignStmt, LetStmt, LetStmtTarget, Binding};
use concrete_ast::types::TypeSpec;
//use concrete_ast::expressions::Value; // Import the missing module
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum VarState {
    Unconsumed,
    Consumed,
    _Borrowed,
    _BorrowedMut,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarInfo {
    ty: String, //TODO Define 'Type' as needed
    depth: usize,
    state: VarState,
}

#[derive(Debug, Clone, PartialEq)]
enum CountResult {
    Zero,
    One,
    MoreThanOne,
}

#[derive(Debug, Clone, Copy)]
struct Appearances {
    consumed: u32,
    write: u32,
    read: u32,
    path: u32,
}


// TODO remove. This are structures translated from Austral
#[allow(dead_code)]
enum Expr {
    NilConstant,
    BoolConstant(bool),
    IntConstant(i32),
    FloatConstant(f64),
    StringConstant(String),
    ConstVar,
    ParamVar(String),
    LocalVar(String),
    FunVar,
    Funcall(Box<Expr>, Vec<Expr>),
    MethodCall(Box<Expr>, Vec<Expr>),
    VarMethodCall(Vec<Expr>),
    FptrCall(Box<Expr>, Vec<Expr>),
    Cast(Box<Expr>, String),
    Comparison(Box<Expr>, Box<Expr>),
    Conjunction(Box<Expr>, Box<Expr>),
    Disjunction(Box<Expr>, Box<Expr>),
    Negation(Box<Expr>),
    IfExpression(Box<Expr>, Box<Expr>, Box<Expr>),
    RecordConstructor(Vec<Expr>),
    UnionConstructor(Vec<Expr>),
    Path { head: Box<Expr>, elems: Vec<Expr> },
    Embed(Vec<Expr>),
    Deref(Box<Expr>),
    SizeOf,
    Borrow(BorrowMode, String),
    ArrayIndex(Box<Expr>),
}

#[allow(dead_code)]
enum BorrowMode {
    ReadBorrow,
    WriteBorrow,
}

impl Appearances {
    fn new(consumed: u32, write: u32, read: u32, path: u32) -> Self {
        Appearances { consumed, write, read, path }
    }

    fn partition(count: u32) -> CountResult {
        match count {
            0 => CountResult::Zero,
            1 => CountResult::One,
            _ => CountResult::MoreThanOne,
        }
    }

    fn zero() -> Self {
        Self::new(0, 0, 0, 0)
    }

    fn consumed_once() -> Self {
        Self::new(1, 0, 0, 0)
    }

    // When borrowed implemented
    fn _read_once() -> Self {
        Self::new(0, 0, 1, 0)
    }

    fn _write_once() -> Self {
        Self::new(0, 1, 0, 0)
    }

    fn path_once() -> Self {
        Self::new(0, 0, 0, 1)
    }

    /* TODO implementation of merge without copying
    fn merge(&mut self, other: &Appearances) {
            self.consumed += other.consumed;
            self.write += other.write;
            self.read += other.read;
            self.path += other.path;
    }*/

    fn merge(&self, other: &Appearances) -> Self {
        Appearances {
            consumed: self.consumed + other.consumed,
            write: self.write + other.write,
            read: self.read + other.read,
            path: self.path + other.path,
        }
    }
    

    fn _merge_list(appearances: Vec<Appearances>) -> Self {
        appearances.into_iter().fold(Self::zero(), |acc, x| acc.merge(&x))
    }
}


#[derive(Debug, Clone)]
struct StateTbl {
    vars: HashMap<String, VarInfo>,
}

impl StateTbl {
    // Initialize with an empty state table
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    // Example of updating the state table
    fn update_info(&mut self, var: & str, info: VarInfo) {
        self.vars.insert(var.to_string(), info);
    }

    /* 
    // Remove a variable from the state table
    fn remove_entry(&mut self, var: &str) {
        self.vars.remove(var);
    }
    */

    fn get_info_mut(&mut self, var: &str) -> Option<&mut VarInfo> {
        if !self.vars.contains_key(var){
            self.vars.insert(var.to_string(), VarInfo{ty: "".to_string(), depth: 0, state: VarState::Unconsumed});
            println!("Variable {} not found in state table. Inserting with default state", var);
        }
        self.vars.get_mut(var)        
    }

    fn get_info(& self, var: &str) -> Option<& VarInfo> {
        self.vars.get(var)        
    }

    // Retrieve a variable's state
    fn _get_state(&mut self, var: &str) -> Option<&VarState> {
        if let Some(info) = self.get_info(var) {
            Some(&info.state)
        } else {
            None
        }
    }

    // Retrieve a variable's state
    fn update_state(&mut self, var: &str, new_state: &VarState){
        let info = self.get_info_mut(var);
        if let Some(info) = info {
            info.state = new_state.clone();
        } 
    }


    fn get_loop_depth(&mut self, name: &str) -> usize {
        let state = self.get_info(name);
        if let Some(state) = state {
            state.depth
        } else {
            0
        }
    }
}















struct LinearityChecker {
    state_tbl: StateTbl,
}

#[allow(dead_code)]
#[allow(unused_variables)]
impl LinearityChecker {
    fn new() -> Self {
        LinearityChecker {
            state_tbl: StateTbl::new(),
        }
    }

    //TODO remove
    /* 
    fn linearity_check(&mut self, program: &FunctionDef) -> Result<(), LinearityError> {
        // Assume Program is a struct that represents the entire program.
        for statement in &program.body {
            self.check_stmt(0, &statement);
        }
        Ok(())
    }
    */
    /* 
    fn check_function(&mut self, function: &FnBody) -> Result<(), LinearityError> {
        // Logic to check linearity within a function
        // This may involve iterating over statements and expressions, similar to OCaml's recursion.
        for basic_block in &function.basic_blocks{
            for statement in &basic_block.statements {
                self.check_stmt(0, &statement)?;
            }
            
        } 
        Ok(())
    }*/

    

    fn consume_once(&mut self, depth: usize, name: &str) -> Result<(), LinearityError> {
        let loop_depth = self.state_tbl.get_loop_depth(name);
        println!("Consuming variable: {} depth {} loop_depth {}", name, depth, loop_depth);
        if depth == self.state_tbl.get_loop_depth(name) {
            self.state_tbl.update_state(name, &VarState::Consumed);
            println!("Consumed variable: {}", name);
            /* 
            let mut state = self.state_tbl.get_state(name);
            if let Some(state) = state {
                state = &VarState::Consumed;
            }
            else{
                //self.state_tbl.update_state(name, VarInfo{"".to_string(), depth, VarState::Unconsumed});
            }*/
            
            Ok(())
        }
        else{
            Err(LinearityError::ConsumedMoreThanOnce { variable: name.to_string()})
        }
    }

    
    
    fn check_expr(&mut self, depth: usize, expr: &Expression) -> Result<(), LinearityError> {
        // Assuming you have a method to get all variable names and types
        //let vars = &mut self.state_tbl.vars; 
        //TODO check if we can avoid cloning
        let vars = self.state_tbl.vars.clone(); 
        for (name, _info) in vars.iter() {
            //self.check_var_in_expr(depth, &name, &info.ty, expr)?;
            self.check_var_in_expr(depth, name, expr)?;
        }
        Ok(())
    }


    fn count_in_statements(&self, name: &str, statements: &Vec<Statement>) -> Appearances {
        statements.iter().map(|stmt| self.count_in_statement(name, stmt)).fold(Appearances::zero(), |acc, x| acc.merge(&x))
    }
    
    fn count_in_statement(&self, name: &str, statement: &Statement) -> Appearances {
        match statement {
            Statement::Let(binding) => {
                // Handle let bindings, possibly involving pattern matching
                self.count_in_expression(name, &binding.value)
            },
            Statement::If(if_stmt) => {
                // Process all components of an if expression
                let cond_apps = self.count_in_expression(name, &if_stmt.value);
                let then_apps = self.count_in_statements(name, &if_stmt.contents);
                let else_apps;
                let else_statements = &if_stmt.r#else;
                if let Some(else_statements) = else_statements {
                    else_apps = self.count_in_statements(name, &else_statements);
                } else {
                    else_apps = Appearances::zero();
                }
                cond_apps.merge(&then_apps).merge(&else_apps)
            },
            Statement::While(while_expr) => {
                let cond= &while_expr.value;
                let block = &while_expr.contents;
                // Handle while loops
                self.count_in_expression(name, cond).merge(&self.count_in_statements(name, block))
            },
            Statement::For(for_expr) => {
                // Handle for loops
                //init, cond, post, block
                let init = &for_expr.init;
                let cond = &for_expr.condition;
                let post = &for_expr.post;
                let block = &for_expr.contents;
                let mut apps = Appearances::zero();
                if let Some(init) = init{
                    if let Some(cond) = cond{
                        if let Some(post) = post{
                            apps = self.count_in_let_statements(name, init).merge(&self.count_in_expression(name, cond)).merge(&self.count_in_assign_statement(name, post)).merge(&self.count_in_statements(name, block))
                        }
                    }
                }
                apps
            },
            /* Alucination of GPT
            Statement::Block(statements) => {
                // Handle blocks of statements
                //statements.iter().map(|stmt| self.count(name, stmt)).fold(Appearances::zero(), |acc, x| acc.merge(&x))
                self.count_in_statements(name, statements)
            },*/
            Statement::Assign(assign_stmt) => {
                // Handle assignments
                self.count_in_assign_statement(name, assign_stmt)
            },
            Statement::Return(return_stmt) => {
                // Handle return statements
                if let Some(value) = &return_stmt.value {
                    self.count_in_expression(name, value)
                } else {
                    Appearances::zero()
                }
            },
            Statement::FnCall(fn_call_op) => {
                // Process function call arguments
                //fn_call_op.target.iter().map(|arg| self.count_in_path_op(name, arg)).fold(Appearances::zero(), |acc, x| acc.merge(&x));
                fn_call_op.args.iter().map(|arg| self.count_in_expression(name, arg)).fold(Appearances::zero(), |acc, x| acc.merge(&x))
            },
            Statement::Match(_) => {
                todo!("do not support match statement")
            },
            //_ => Appearances::zero(),
        } 
    }

    fn count_in_assign_statement(&self, name: &str, assign_stmt: &AssignStmt) -> Appearances {
        match assign_stmt {
            AssignStmt { target, derefs, value, span } => {
                // Handle assignments
                let ret = self.count_in_path_op(name, target);
                ret.merge(&self.count_in_expression(name, value));                
                ret
            },
        }
    }

    fn count_in_path_op(&self, name: &str, path_op: &PathOp) -> Appearances {
        let apps:   Appearances;
        if name == path_op.first.name{
            apps = Appearances::path_once();
        }
        else{
            apps = Appearances::zero();
        }
        apps
    }

    fn count_in_let_statements(&self, name: &str, let_stmt: &LetStmt) -> Appearances {
        match let_stmt {
            LetStmt { is_mutable, target, value, span } => {
                // Handle let bindings, possibly involving pattern matching
                self.count_in_expression(name, value)
            },
        }
    }

    fn count_in_expression(&self, name: &str, expr: &Expression) -> Appearances {
        match expr {
            Expression::Value(value_expr, _) => {
                // Handle value expressions, typically constant or simple values
                //Appearances::zero()
                match value_expr {
                    ValueExpr::ValueVar(ident, _) => {
                        if name == ident.name {
                            Appearances::consumed_once()
                        } else {
                            Appearances::zero()
                        }
                    },
                    ValueExpr::Path(path) => {
                        //path.first.name == name;
                        Appearances::zero()
                    },
                    _ => Appearances::zero(),
                }
            },
            Expression::FnCall(fn_call_op) => {
                // Process function call arguments
                fn_call_op.args.iter().map(|arg| self.count_in_expression(name, arg)).fold(Appearances::zero(), |acc, x| acc.merge(&x))
            },
            Expression::Match(match_expr) => todo!("do not support match expression"),            
            /* 
            Expression::Match(match_expr) => {
                // Handle match arms
                match_expr.variants.iter().map(|(_, expr)| self.count(name, expr)).fold(Appearances::zero(), |acc, x| acc.merge(&x))
            },*/
            Expression::If(if_expr) => {
                // Process all components of an if expression
                let cond_apps = self.count_in_expression(name, &if_expr.value);
                let then_apps = self.count_in_statements(name, &if_expr.contents);
                //let else_apps = if_expr.else.as_ref().map(|e| self.count(name, e)).unwrap_or_default();
                cond_apps.merge(&then_apps);
                if let Some(else_block) = &if_expr.r#else {
                    let else_apps = self.count_in_statements(name, else_block);
                    cond_apps.merge(&then_apps).merge(&else_apps);
                }
                cond_apps
            },
            Expression::UnaryOp(_, expr) => {
                // Unary operations likely don't change the count but process the inner expression
                self.count_in_expression(name, expr)
            },
            Expression::BinaryOp(left, _, right) => {
                // Handle binary operations by processing both sides
                self.count_in_expression(name, left).merge(&&self.count_in_expression(name, right))
            },
            Expression::StructInit(struct_init_expr) => {
                // Handle struct initialization
                struct_init_expr.fields.iter().map(|(_, expr)| self.count_struct_init(name, expr)).fold(Appearances::zero(), |acc, x| acc.merge(&x))
            },
            Expression::ArrayInit(array_init_expr) => {
                // Handle array initializations
                array_init_expr.values.iter().map(|expr| self.count_in_expression(name, expr)).fold(Appearances::zero(), |acc, x| acc.merge(&x))
            },
            Expression::Deref(expr, _) | Expression::AsRef(expr, _, _) | Expression::Cast(expr, _, _) => {
                // Deref, AsRef, and Cast are handled by just checking the inner expression
                self.count_in_expression(name, expr)
            },
            // Add more cases as necessary based on the Expression types you expect
        }
    }


    fn count_struct_init(&self, name: &str, struct_init: &StructInitField) -> Appearances {
        println!("Checking struct init: {:?}", struct_init);
        self.count_in_expression(name, &struct_init.value)
    }

    fn check_stmt_let(&mut self, depth: usize, binding: &LetStmt) -> Result<(), LinearityError> {
        // Handle let bindings, possibly involving pattern matching
        let LetStmt { is_mutable, target, value, span } = binding;
        match target {
            LetStmtTarget::Simple { name, r#type } => {
                match r#type {
                    TypeSpec::Simple{ name: variable_type , qualifiers, span} => {
                        self.state_tbl.update_info(&name.name, VarInfo{ty: variable_type.name.clone(), depth, state: VarState::Unconsumed});
                    },                    
                    TypeSpec::Generic { name: variable_type, qualifiers, type_params, span } =>{
                        self.state_tbl.update_info(&name.name, VarInfo{ty: variable_type.name.clone(), depth, state: VarState::Unconsumed});
                    },
                    TypeSpec::Array { of_type, size, qualifiers, span } => {
                        let array_type = "Array<".to_string() + &of_type.get_name() + ">";
                        self.state_tbl.update_info(&name.name, VarInfo{ty: array_type, depth, state: VarState::Unconsumed});
                    },
                }
                self.check_var_in_expr(depth, &name.name, value)
            },
            LetStmtTarget::Destructure(bindings) => {
                for binding in bindings {
                    self.check_bindings(depth, binding)?;
                }
                Ok(())
            },
        }
    }

    fn check_bindings(&mut self, depth: usize, binding: &Binding) -> Result<(), LinearityError> {
        // TODO Do something with the bindings        
        println!("TODO implement Checking bindings: {:?}", binding);
        Ok(())
    }

    fn check_stmts(&mut self, depth: usize, stmts: &Vec<Statement>) -> Result<(), LinearityError> {
        for stmt in stmts {
            self.check_stmt(depth, stmt)?;
        }
        Ok(())
    }

    fn check_stmt(&mut self, depth: usize, stmt: &Statement) -> Result<(), LinearityError> {
        match stmt {
            /* 
            Statement::Expression(expr) => {
                // Handle expressions (e.g., variable assignments, function calls)
                self.check_expr(depth, expr)
            },*/
            Statement::Let(binding) => {
                // Handle let bindings, possibly involving pattern matching
                self.check_stmt_let(depth, binding)
            },
            //Statement::If(cond, then_block, else_block) => {
            Statement::If(if_stmt) => {
                // Handle conditional statements
                self.check_expr(depth, &if_stmt.value)?;
                self.check_stmts(depth + 1, &if_stmt.contents)?;
                if let Some(else_block) = &if_stmt.r#else {
                    self.check_stmts(depth + 1, &else_block)?;
                }
                Ok(())
            },
            //Statement::While(cond, block) => {
                Statement::While(while_stmt) => {
                // Handle while loops
                self.check_expr(depth, &while_stmt.value)?;
                self.check_stmts(depth + 1, &while_stmt.contents)
            },
            //Statement::For(init, cond, post, block) => {
            Statement::For(for_stmt) => {
                // Handle for loops
                if let Some(init) = &for_stmt.init   {
                    self.check_stmt_let(depth, init)?;
                }
                if let Some(condition) = &for_stmt.condition   {
                    self.check_expr(depth, condition)?;
                }
                if let Some(post) = &for_stmt.post {
                    //TODO check assign statement
                    //self.check_stmt_assign(depth, post)?;
                }
                self.check_stmts(depth + 1, &for_stmt.contents)
            },
            Statement::Assign(assign_stmt) => {
                // Handle assignments
                let AssignStmt { target, derefs, value, span } = assign_stmt;
                println!("Checking assignment: {:?}", assign_stmt);
                self.check_path_opt(depth, target)?;
                //self.check_expr(depth, &self.path_op_to_expression(target))?;
                self.check_expr(depth, value)
            },
            Statement::Return(return_stmt) => {
                if let Some(value) = &return_stmt.value {
                    self.check_expr(depth, value)
                } else {
                    Ok(())
                }
            },
            Statement::FnCall(fn_call_op) => {
                // Process function call arguments
                for arg in &fn_call_op.args {
                    self.check_expr(depth, arg)?;
                }
                Ok(())
            },
            Statement::Match(_) => {
                println!("Skipping linearity check for statement type: \n{:?}", stmt);
                todo!()
            }           
        }
    }
    
    fn check_path_opt(&mut self, depth: usize, path_op: &PathOp) -> Result<(), LinearityError> {
        println!("Checking path: {:?}", path_op);
        //let var_expression = Value::new(path_op.first.clone(), path_op.span); // Use the imported module    
        let var_expression = Expression::Value(ValueExpr::ValueVar(path_op.first.clone(), path_op.span), path_op.span);         
        self.check_var_in_expr(depth, &path_op.first.name, &var_expression)        
    }
    /*
    fn path_op_to_expression(&self, path_op: &PathOp) -> Expression {
        // Convert the first identifier part of the path into an Expression
        let mut expr = Expression::Variable(path_op.first.clone());

        // Process additional path segments
        for segment in &path_op.extra {
            match segment {
                PathSegment::Field(field) => {
                    expr = Expression::Field(Box::new(expr), field.clone());
                },
                PathSegment::Index(index) => {
                    // Assuming index is an Expression
                    expr = Expression::Index(Box::new(expr), Box::new(Expression::Variable(index.clone())));
                },
                // Add other cases as necessary
            }
        }

        expr
    }

    fn build_complex_expression_from_path(&self, components: &[PathComponent]) -> Expression {
        // Construct a complex expression from path components
        // This is just a placeholder, real implementation will depend on your specific case
        Expression::Variable(components.iter().map(|c| c.to_string()).collect::<Vec<_>>().join("."))
    }*/

    fn check_var_in_expr(&mut self, depth: usize, name: &str, expr: &Expression) -> Result<(), LinearityError> {
        
        let info = self.state_tbl.get_info(name); // Assume default state
        if let Some(info) = info{
            //Only checks Linearity for types of name Linear
            // TODO improve this approach
            if info.ty == *"Linear".to_string(){
                let state = &info.state;
                let apps = self.count_in_expression(name, expr); // Assume count function implementation
                let Appearances { consumed, write, read, path } = apps;            
                //println!("Checking variable: {} with state: {:?} and appearances: {:?} in expression {:?}", name, state, apps, expr);
                println!("Checking state_tbl variable: {}: {:?} {:?} in expression {:?}", name, info, apps, expr);
                match (state, Appearances::partition(consumed), Appearances::partition(write), Appearances::partition(read), Appearances::partition(path)) {
                  /*(        State            Consumed           WBorrow             RBorrow           Path      )
                    (* ------------------|-------------------|-----------------|------------------|----------------)*/
                    // Not yet consumed, and at most used through immutable borrows or path reads.
                    (VarState::Unconsumed, CountResult::Zero, CountResult::Zero,                 _,                 _) => Ok(()),
                    // Not yet consumed, borrowed mutably once, and nothing else.
                    (VarState::Unconsumed, CountResult::Zero, CountResult::One,  CountResult::Zero, CountResult::Zero) => Ok(()),
                    // Not yet consumed, borrowed mutably, then either borrowed immutably or accessed through a path.
                    (VarState::Unconsumed, CountResult::Zero, CountResult::One,                  _,                 _) => Err(LinearityError::BorrowedMutUsed { variable: name.to_string() }),
                    // Not yet consumed, borrowed mutably more than once.
                    (VarState::Unconsumed, CountResult::Zero, CountResult::MoreThanOne,          _,                 _) => Err(LinearityError::BorrowedMutMoreThanOnce { variable: name.to_string() }),
                    // Not yet consumed, consumed once, and nothing else. Valid IF the loop depth matches.
                    (VarState::Unconsumed, CountResult::One,   CountResult::Zero, CountResult::Zero, CountResult::Zero) => self.consume_once(depth, name),
                    // Not yet consumed, consumed once, then either borrowed or accessed through a path.
                    (VarState::Unconsumed, CountResult::One,                   _,                 _,                 _) => Err(LinearityError::ConsumedAndUsed { variable: name.to_string() }),
                    // Not yet consumed, consumed more than once.
                    (VarState::Unconsumed, CountResult::MoreThanOne,           _,                 _,                 _) => Err(LinearityError::ConsumedMoreThanOnce { variable: name.to_string() }),
                    // Read borrowed, and at most accessed through a path.
                    (VarState::_Borrowed, CountResult::Zero, CountResult::Zero, CountResult::Zero, _) => Ok(()),
                    // Read borrowed, and either consumed or borrowed again.
                    (VarState::_Borrowed,                    _,                 _,                 _,                 _) => Err(LinearityError::ReadBorrowedAndUsed { variable: name.to_string() }),
                    // Write borrowed, unused.
                    (VarState::_BorrowedMut,  CountResult::Zero, CountResult::Zero, CountResult::Zero, CountResult::Zero) => Ok(()),
                    // Write borrowed, used in some way.
                    (VarState::_BorrowedMut,                 _,                 _,                 _,                  _) => Err(LinearityError::WriteBorrowedAndUsed { variable: name.to_string() }),
                    // Already consumed, and unused.
                    (VarState::Consumed,     CountResult::Zero, CountResult::Zero, CountResult::Zero, CountResult::Zero) => Ok(()),
                    // Already consumed, and used in some way.
                    (VarState::Consumed,                     _,                 _,                 _,                 _) => Err(LinearityError::AlreadyConsumedAndUsed { variable: name.to_string() }),
                }
            }
            else{
                //Only checks Linearity for types of name Linear
                Ok(())
            }
        }
        else {
            Err(LinearityError::VariableNotFound { variable: name.to_string()})
        }
    }
    /*
    fn check_var_in_expr(&mut self, depth: u32, name: &str, state: &VarState, expr: &Expression) -> Result<(), LinearityError> {
        let apps = self.count_in_expression(name, expr); // Assume count function implementation
        let Appearances { consumed, write, read, path } = apps;
    
        //let state = self.state_tbl.get_state(name).unwrap_or(&VarState::Unconsumed); // Assume default state
        let state = self.state_tbl.get_state(name);// Assume default state
        if let Some(state) = state{
            println!("Checking variable: {} with state: {:?} and appearances: {:?}", name, state, apps);
            match (state, Appearances::partition(consumed), Appearances::partition(write), Appearances::partition(read), Appearances::partition(path)) {
                /*(        State            Consumed           WBorrow             RBorrow           Path      )
                (* ------------------|-------------------|-----------------|------------------|----------------)*/
                //(VarState::Unconsumed, CountResult::Zero, CountResult::Zero,                _,                 _) => Ok(state_tbl.clone()),
                //(VarState::Unconsumed, CountResult::Zero, CountResult::One, CountResult::Zero, CountResult::Zero) => Ok(state_tbl.clone()),
                (VarState::Unconsumed, CountResult::Zero, CountResult::Zero,                 _,                 _) => Ok(()),
                (VarState::Unconsumed, CountResult::Zero, CountResult::One, CountResult::Zero, CountResult::Zero) => Ok(()),
                (VarState::Unconsumed, CountResult::Zero, CountResult::MoreThanOne,          _,                 _) =>
                    Err(LinearityError::BorrowedMutMoreThanOnce { variable: name.to_string() }),
                (VarState::Unconsumed, CountResult::One,                         _,          _,                 _) =>
                    Err(LinearityError::ConsumedAndUsed { variable: name.to_string() }),
                (VarState::Unconsumed, CountResult::MoreThanOne,                 _,          _,                 _) =>
                    Err(LinearityError::ConsumedMoreThanOnce { variable: name.to_string() }),
                (VarState::Borrowed,                  _,                         _,          _,                 _) =>
                    Err(LinearityError::ReadBorrowedAndUsed { variable: name.to_string() }),
                (VarState::BorrowedMut,               _,                         _,          _,                 _) =>
                    Err(LinearityError::WriteBorrowedAndUsed { variable: name.to_string() }),
                (VarState::Consumed,                  _,                         _,          _,                 _) =>
                    Err(LinearityError::AlreadyConsumedAndUsed { variable: name.to_string() }),
                _ => Err(LinearityError::UnhandledStateOrCount { variable: name.to_string() }),
            }
        }
        else {
            Err(LinearityError::VariableNotFound { variable: name.to_string() })
        }
        
    }*/
    
}



//#[cfg(feature = "linearity")]
#[allow(unused_variables)]
//pub fn linearity_check_program(program_ir: &FunctionDef, session: &Session) ->  Result<String, LinearityError> {
pub fn linearity_check_program(programs: &Vec<(PathBuf, String, Program)>, session: &Session) ->  Result<String, LinearityError> {
    println!("Starting linearity check");
    let mut checker = LinearityChecker::new();
    for (path, name, program) in programs {
        println!("Checking linearity for program: {}", name);
        for module in &program.modules {
            println!("Checking linearity for module: {}", module.name.name);
            for module_content in &module.contents {
                match module_content {
                    ModuleDefItem::Function(function) => {
                        //println!("Checking linearity for function: {:?}", function);
                        //checker.check_function(&function)?;
                        for statement in &function.body {
                            //println!("Checking linearity for function body: {:?}", function.body);                        
                            checker.check_stmt(0, statement)?;
                        }
                        println!("Finished checking linearity for function: {} {:?}", function.decl.name.name, checker.state_tbl);
                        //checker.linearity_check(&function)?;
                    },
                    ModuleDefItem::FunctionDecl(function_decl) => 
                    {
                        println!("Skipping linearity check for FunctionDecl: {:?}", module_content);
                    },
                    ModuleDefItem::Module(module) => 
                    { 
                        println!("Skipping linearity check for Module: {:?}", module_content);
                    },
                    ModuleDefItem::Struct(struc) => 
                    { 
                        //println!("Skipping linearity check for Struct: {:?}", module_content);
                        //checker.
                        checker.state_tbl.update_info(&struc.name.name, VarInfo{ty: "Struct".to_string(), depth: 0, state: VarState::Unconsumed});
                    },
                    ModuleDefItem::Enum(_) => 
                    { 
                        println!("Skipping linearity check for Enum: {:?}", module_content);
                    },
                    ModuleDefItem::Constant(_) => 
                    { 
                        println!("Skipping linearity check for Constant: {:?}", module_content);
                    },
                    ModuleDefItem::Union(_) => 
                    { 
                        println!("Skipping linearity check for Uinon: {:?}", module_content);
                    },
                    ModuleDefItem::Type(_) => 
                    { 
                        println!("Skipping linearity check for module content: {:?}", module_content);
                    },
                    /*_ => 
                    { 
                        println!("Skipping linearity check for module content: {:?}", module_content);
                        ()
                    },*/                    
                }                
            }
        }
    }
    Ok("OK".to_string())
}
