use crate::ast::{AstStatement, SourceRange};
use crate::index::const_expressions::ConstExpression;
use crate::index::Index;
use crate::resolver::const_evaluator::{evaluate_constants, UnresolvableConstant};
use crate::resolver::tests::parse;

const EMPTY: Vec<UnresolvableConstant> = vec![];

///locally overwerite assert_eq to assert the Debug-Equality
macro_rules! debug_assert_eq {
    ($left:expr, $right:expr) => {
        assert_eq!(format!("{:#?}", $left), format!("{:#?}", $right))
    };
}

macro_rules! global {
    ($index:expr, $name:expr) => {
        $index
            .find_global_variable($name)
            .unwrap()
            .initial_value
            .unwrap()
    };
}

fn find_member_value<'a>(index: &'a Index, pou: &str, reference: &str) -> Option<&'a AstStatement> {
    index.find_member(pou, reference).and_then(|it| {
        index
            .get_const_expressions()
            .maybe_get_constant_statement(&it.initial_value)
    })
}

fn find_connstant_value<'a>(index: &'a Index, reference: &str) -> Option<&'a AstStatement> {
    index.find_global_variable(reference).and_then(|it| {
        index
            .get_const_expressions()
            .maybe_get_constant_statement(&it.initial_value)
    })
}

fn create_int_literal(v: i128) -> AstStatement {
    AstStatement::LiteralInteger {
        value: v,
        id: 0,
        location: SourceRange::undefined(),
    }
}

fn create_string_literal(v: &str, wide: bool) -> AstStatement {
    AstStatement::LiteralString {
        value: v.to_string(),
        is_wide: wide,
        id: 0,
        location: SourceRange::undefined(),
    }
}

fn create_real_literal(v: f64) -> AstStatement {
    AstStatement::LiteralReal {
        value: format!("{:}", v),
        id: 0,
        location: SourceRange::undefined(),
    }
}

fn create_bool_literal(v: bool) -> AstStatement {
    AstStatement::LiteralBool {
        value: v,
        id: 0,
        location: SourceRange::undefined(),
    }
}

#[test]
fn const_references_to_int_compile_time_evaluation() {
    // GIVEN some INT index used as initializers
    let (_, index) = parse(
        "VAR_GLOBAL CONSTANT
            iX : INT := 4;
            rX : LREAL := 4.2;
            iY : INT := iX;
            rY : LREAL := iX;
            iZ : INT := iY;
            rZ : LREAL := rY;
        END_VAR
        
        VAR_GLOBAL CONSTANT
            a : INT := iX;
            b : INT := iY;
            c : INT := iZ;
            d : LREAL := rX;
            e : LREAL := rY;
            f : LREAL := rZ;
        END_VAR
        ",
    );

    // WHEN compile-time evaluation is applied
    let (index, unresolvable) = evaluate_constants(index);

    // THEN a to f got their correct initial-literals
    debug_assert_eq!(EMPTY, unresolvable);
    debug_assert_eq!(
        &create_int_literal(4),
        find_connstant_value(&index, "a").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(4),
        find_connstant_value(&index, "b").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(4),
        find_connstant_value(&index, "c").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(4.2),
        find_connstant_value(&index, "d").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(4.0),
        find_connstant_value(&index, "e").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(4.0),
        find_connstant_value(&index, "f").unwrap()
    );
}

#[test]
fn local_const_references_to_int_compile_time_evaluation() {
    // GIVEN some INT index used as initializers
    let (_, index) = parse(
        "
        PROGRAM prg 
            VAR CONSTANT
                iX : INT := 4;
                rX : LREAL := 4.2;
           END_VAR
        END_PROGRAM

        VAR_GLOBAL CONSTANT
            a : INT := prg.iX;
            b : LREAL := prg.rX;
       END_VAR
        ",
    );

    // WHEN compile-time evaluation is applied
    let (index, unresolvable) = evaluate_constants(index);

    // THEN a to f got their correct initial-literals
    debug_assert_eq!(EMPTY, unresolvable);
    debug_assert_eq!(
        &create_int_literal(4),
        find_connstant_value(&index, "a").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(4.2),
        find_connstant_value(&index, "b").unwrap()
    );
}

#[test]
fn local_const_references_to_int_compile_time_evaluation_uses_correct_scopes() {
    // GIVEN some global and local constants
    let (_, index) = parse(
        "
        VAR_GLOBAL CONSTANT
            a : INT := 5;
        END_VAR

        VAR_GLOBAL
            g : INT := a; //should be 5
            h : INT := prg.a; // should be 4
        END_VAR

        PROGRAM prg 
            VAR CONSTANT
                a : INT := 4;
            END_VAR

            VAR_INPUT
                v : INT := a; //should be 4
            END_VAR
        END_PROGRAM
        ",
    );

    // WHEN compile-time evaluation is applied
    let (index, unresolvable) = evaluate_constants(index);
    debug_assert_eq!(EMPTY, unresolvable);

    // THEN g should resolve its inital value to global 'a'
    debug_assert_eq!(
        &create_int_literal(5),
        find_connstant_value(&index, "g").unwrap()
    );
    // THEN h should resolve its inital value to 'prg.a'
    debug_assert_eq!(
        &create_int_literal(4),
        find_connstant_value(&index, "h").unwrap()
    );
    // AND prg.v should resolve its initial value to 'prg.a'
    debug_assert_eq!(
        &create_int_literal(4),
        find_member_value(&index, "prg", "v").unwrap()
    );
}

#[test]
fn non_const_references_to_int_compile_time_evaluation() {
    // GIVEN some global consts
    // AND some NON-constants
    let (_, index) = parse(
        "VAR_GLOBAL CONSTANT
            iX : INT := 2;
        END_VAR

        VAR_GLOBAL
            a : INT := 3;
            b : INT := 4;
        END_VAR
        
        VAR_GLOBAL CONSTANT
            ok      : INT := iX;
            nok_a   : INT := iX + a;
            nok_b   : INT := iX + b;

            temp        : INT := a;
            incomplete  : INT := temp;
        END_VAR
        ",
    );

    // WHEN compile-time evaluation is applied
    let (index, unresolvable) = evaluate_constants(index);

    // THEN a to f got their correct initial-literals
    debug_assert_eq!(
        &create_int_literal(2),
        find_connstant_value(&index, "ok").unwrap()
    );

    debug_assert_eq!(
        vec![
            UnresolvableConstant::new(global!(index, "nok_a"), "'a' is no const reference"),
            UnresolvableConstant::new(global!(index, "nok_b"), "'b' is no const reference"),
            UnresolvableConstant::new(global!(index, "temp"), "'a' is no const reference"),
            UnresolvableConstant::incomplete_initialzation(&global!(index, "incomplete")), //this one is fine, but one depency cannot be resolved
        ],
        unresolvable
    );
}

#[test]
fn prg_members_initials_compile_time_evaluation() {
    // GIVEN some member variables with const initializers
    let (_, index) = parse(
        "
        VAR_GLOBAL CONSTANT
            TWO : INT := 2;
            FIVE : INT := TWO * 2 + 1;
            C_STR : STRING := 'hello world';
        END_VAR

        PROGRAM plc_prg
            VAR_INPUT
                a : INT := TWO;
                b : INT := TWO + 4;
                c : INT := FIVE;
                str : STRING := C_STR;
            END_VAR
        END_PROGRAM
       END_VAR
        ",
    );

    // WHEN compile-time evaluation is applied
    let (index, unresolvable) = evaluate_constants(index);

    // THEN everything got resolved
    debug_assert_eq!(EMPTY, unresolvable);
    // AND the program-members got their correct initial-literals
    debug_assert_eq!(
        &create_int_literal(2),
        find_member_value(&index, "plc_prg", "a").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(6),
        find_member_value(&index, "plc_prg", "b").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(5),
        find_member_value(&index, "plc_prg", "c").unwrap()
    );
    debug_assert_eq!(
        &create_string_literal("hello world", false),
        find_member_value(&index, "plc_prg", "str").unwrap()
    );
}

#[test]
fn const_references_to_negative_reference() {
    // GIVEN some INT index used as initializers
    let (_, index) = parse(
        "VAR_GLOBAL CONSTANT
            iX : INT := 4;
            rX : LREAL := 4.2;
        END_VAR
        
        VAR_GLOBAL CONSTANT
            a : INT := -iX;
            b : LREAL := -rX;
            c : INT := -5;
       END_VAR
        ",
    );

    // WHEN compile-time evaluation is applied
    let (index, unresolvable) = evaluate_constants(index);

    // THEN a,b,and c got their correct initial-literals
    debug_assert_eq!(EMPTY, unresolvable);
    debug_assert_eq!(
        &create_int_literal(-4),
        find_connstant_value(&index, "a").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(-4.2),
        find_connstant_value(&index, "b").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(-5),
        find_connstant_value(&index, "c").unwrap()
    );
}

#[test]
fn const_references_to_int_additions_compile_time_evaluation() {
    // GIVEN some INT index used as initializers
    let (_, index) = parse(
        "VAR_GLOBAL CONSTANT
            iX : INT := 4;
            rX : LREAL := 4.2;
            iY : INT := iX;
            rY : LREAL := iX;
            iZ : INT := iY + 7;
            rZ : LREAL := rY + 7.7;
        END_VAR
        
        VAR_GLOBAL CONSTANT
            a : INT := iX;
            b : INT := iY;
            c : INT := iZ;
            d : LREAL := rX;
            e : LREAL := rY;
            f : LREAL := rZ;
        END_VAR
        ",
    );

    // WHEN compile-time evaluation is applied
    let (index, unresolvable) = evaluate_constants(index);

    // THEN a,b,and c got their correct initial-literals
    debug_assert_eq!(EMPTY, unresolvable);
    debug_assert_eq!(
        &create_int_literal(4),
        find_connstant_value(&index, "a").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(4),
        find_connstant_value(&index, "b").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(11),
        find_connstant_value(&index, "c").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(4.2),
        find_connstant_value(&index, "d").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(4.0),
        find_connstant_value(&index, "e").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(11.7),
        find_connstant_value(&index, "f").unwrap()
    );
}

#[test]
fn const_references_to_int_subtractions_compile_time_evaluation() {
    // GIVEN some INT index used as initializers
    let (_, index) = parse(
        "VAR_GLOBAL CONSTANT
            iX : INT := 4;
            rX : LREAL := 4.2;
            iY : INT := iX;
            rY : LREAL := iX;
            iZ : INT := iY - 7;
            rZ : LREAL := rY - 7.7;
        END_VAR
        
        VAR_GLOBAL CONSTANT
            a : INT := iX;
            b : INT := iY;
            c : INT := iZ;
            d : LREAL := rX;
            e : LREAL := rY;
            f : LREAL := rZ;
        END_VAR
        ",
    );

    // WHEN compile-time evaluation is applied
    let (index, unresolvable) = evaluate_constants(index);

    // THEN a,b,and c got their correct initial-literals
    debug_assert_eq!(EMPTY, unresolvable);
    debug_assert_eq!(
        &create_int_literal(4),
        find_connstant_value(&index, "a").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(4),
        find_connstant_value(&index, "b").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(-3),
        find_connstant_value(&index, "c").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(4.2),
        find_connstant_value(&index, "d").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(4.0),
        find_connstant_value(&index, "e").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(-3.7),
        find_connstant_value(&index, "f").unwrap()
    );
}

#[test]
fn const_references_to_int_multiplications_compile_time_evaluation() {
    // GIVEN some INT index used as initializers
    let (_, index) = parse(
        "VAR_GLOBAL CONSTANT
            iX : INT := 4;
            rX : LREAL := 4.2;
            iY : INT := iX;
            rY : LREAL := iX;
            iZ : INT := iY * 7;
            rZ : LREAL := rY * 7.7;
        END_VAR
        
        VAR_GLOBAL CONSTANT
            a : INT := iX;
            b : INT := iY;
            c : INT := iZ;
            d : LREAL := rX;
            e : LREAL := rY;
            f : LREAL := rZ;
        END_VAR
        ",
    );

    // WHEN compile-time evaluation is applied
    let (index, unresolvable) = evaluate_constants(index);

    // THEN a,b,and c got their correct initial-literals
    debug_assert_eq!(EMPTY, unresolvable);
    debug_assert_eq!(
        &create_int_literal(4),
        find_connstant_value(&index, "a").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(4),
        find_connstant_value(&index, "b").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(28),
        find_connstant_value(&index, "c").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(4.2),
        find_connstant_value(&index, "d").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(4.0),
        find_connstant_value(&index, "e").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(30.8),
        find_connstant_value(&index, "f").unwrap()
    );
}

#[test]
fn const_references_to_int_division_compile_time_evaluation() {
    // GIVEN some INT index used as initializers
    let (_, index) = parse(
        "VAR_GLOBAL CONSTANT
            iX : INT := 40;
            rX : LREAL := 40.2;
            iY : INT := iX;
            rY : LREAL := iX;
            iZ : INT := iY / 7;
            rZ : LREAL := rY / 7.7;
        END_VAR
        
        VAR_GLOBAL CONSTANT
            a : INT := iX;
            b : INT := iY;
            c : INT := iZ;
            d : LREAL := rX;
            e : LREAL := rY;
            f : LREAL := rZ;
        END_VAR
        ",
    );

    // WHEN compile-time evaluation is applied
    let (index, unresolvable) = evaluate_constants(index);

    // THEN a,b,and c got their correct initial-literals
    debug_assert_eq!(EMPTY, unresolvable);
    debug_assert_eq!(
        &create_int_literal(40),
        find_connstant_value(&index, "a").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(40),
        find_connstant_value(&index, "b").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(5),
        find_connstant_value(&index, "c").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(40.2),
        find_connstant_value(&index, "d").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(40.0),
        find_connstant_value(&index, "e").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(40_f64 / 7.7),
        find_connstant_value(&index, "f").unwrap()
    );
}

#[test]
fn const_references_int_float_type_behavior_evaluation() {
    // GIVEN some INT index used as initializers
    let (_, index) = parse(
        "VAR_GLOBAL CONSTANT
            // INT - INT
            int_plus_int : INT := 3 + 1;
            int_minus_int : INT := 3 - 1;
            int_mul_int : INT := 3 * 2;
            int_div_int : INT := 5 / 2;
            int_mod_int : INT := 5 MOD 2;
            int_eq_int : INT := 5 = 5;
            int_neq_int : INT := 5 <> 5;
            int_g_int : INT := 5 > 5;
            int_ge_int : INT := 5 >= 5;
            int_l_int : INT := 5 < 5;
            int_le_int : INT := 5 <= 5;

            // INT - REAL
            int_plus_real : REAL := 3 + 1.1;
            int_minus_real : REAL := 3 - 1.1;
            int_mul_real : REAL := 3 * 1.1;
            int_div_real : REAL := 5 / 2.1;
            int_mod_real : REAL := 5 MOD 2.1;
            int_eq_real : BOOL := 5 = 2.1;
            int_neq_real : BOOL := 5 <> 2.1;
            int_g_real : BOOL := 5 > 5.0;
            int_ge_real : BOOL := 5 >= 5.0;
            int_l_real : BOOL := 5 < 5.0;
            int_le_real : BOOL := 5 <= 5.0;

            // REAL - INT
            real_plus_int : REAL := 3.3 + 1;
            real_minus_int : REAL := 3.3 - 1;
            real_mul_int : REAL := 3.3 * 2;
            real_div_int : REAL := 5.2 / 2;
            real_mod_int : REAL := 5.2 MOD 2;
            real_eq_int : BOOL := 5.2 = 2;
            real_neq_int : BOOL := 5.2 <> 2;
            real_g_int : BOOL := 5.0 > 5;
            real_ge_int : BOOL := 5.0 >= 5;
            real_l_int : BOOL := 5.0 < 5;
            real_le_int : BOOL := 5.0 <= 5;

            // REAL - REAL
            real_plus_real : REAL := 3.3 + 1.1;
            real_minus_real : REAL := 3.3 - 1.1;
            real_mul_real : REAL := 3.3 * 1.1;
            real_div_real : REAL := 5.3 / 2.1;
            real_mod_real : REAL := 5.3 MOD 2.1;
            real_eq_real : REAL := 5.3 = 2.1;
            real_neq_real : REAL := 5.3 <> 2.1;
            real_g_real : BOOL := 5.0 > 5.0;
            real_ge_real : BOOL := 5.0 >= 5.0;
            real_l_real : BOOL := 5.0 < 5.0;
            real_le_real : BOOL := 5.0 <= 5.0;

            //BOOL - BOOL
            _true_ : BOOL := TRUE;
            _false_ : BOOL := FALSE;
            bool_and_bool : BOOL := _true_ AND _true_;
            bool_or_bool : BOOL := _true_ OR _false_;
            bool_xor_bool : BOOL := _true_ XOR _true_;
            not_bool : BOOL := NOT _true_;
        END_VAR
        ",
    );

    // WHEN compile-time evaluation is applied
    let (index, _) = evaluate_constants(index);

    // THEN some type mixed comparisons could not be resolved (note that real == real or real <> real also dont work)
    let mut expected = vec![
        "real_eq_real",
        "real_neq_real",
        "int_eq_real",
        "int_neq_real",
        "real_eq_int",
        "real_neq_int",
        "int_g_real",
        "int_ge_real",
        "int_l_real",
        "int_le_real",
        "real_g_int",
        "real_ge_int",
        "real_l_int",
        "real_le_int",
        "real_g_real",
        "real_ge_real",
        "real_l_real",
        "real_le_real",
    ];
    expected.sort_unstable();

    let mut unresolvable: Vec<&str> = index
        .get_globals()
        .values()
        .filter(|it| {
            let const_expr = index
                .get_const_expressions()
                .find_const_expression(it.initial_value.as_ref().unwrap());
            matches!(const_expr, Some(ConstExpression::Unresolvable { .. }))
        })
        .map(|it| it.get_qualified_name())
        .collect();
    unresolvable.sort_unstable();
    debug_assert_eq!(expected, unresolvable);

    //
    // INT - INT
    debug_assert_eq!(
        &create_int_literal(4),
        find_connstant_value(&index, "int_plus_int").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(2),
        find_connstant_value(&index, "int_minus_int").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(6),
        find_connstant_value(&index, "int_mul_int").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(2),
        find_connstant_value(&index, "int_div_int").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(5 % 2),
        find_connstant_value(&index, "int_mod_int").unwrap()
    );
    debug_assert_eq!(
        &create_bool_literal(true),
        find_connstant_value(&index, "int_eq_int").unwrap()
    );
    debug_assert_eq!(
        &create_bool_literal(false),
        find_connstant_value(&index, "int_neq_int").unwrap()
    );
    debug_assert_eq!(
        &create_bool_literal(false),
        find_connstant_value(&index, "int_g_int").unwrap()
    );
    debug_assert_eq!(
        &create_bool_literal(true),
        find_connstant_value(&index, "int_ge_int").unwrap()
    );
    debug_assert_eq!(
        &create_bool_literal(false),
        find_connstant_value(&index, "int_l_int").unwrap()
    );
    debug_assert_eq!(
        &create_bool_literal(true),
        find_connstant_value(&index, "int_le_int").unwrap()
    );
    // INT - REAL
    debug_assert_eq!(
        &create_real_literal(4.1),
        find_connstant_value(&index, "int_plus_real").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(3.0 - 1.1),
        find_connstant_value(&index, "int_minus_real").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(3.0 * 1.1),
        find_connstant_value(&index, "int_mul_real").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(5.0 / 2.1),
        find_connstant_value(&index, "int_div_real").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(5.0 % 2.1),
        find_connstant_value(&index, "int_mod_real").unwrap()
    );
    // REAL - INT
    debug_assert_eq!(
        &create_real_literal(4.3),
        find_connstant_value(&index, "real_plus_int").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(2.3),
        find_connstant_value(&index, "real_minus_int").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(6.6),
        find_connstant_value(&index, "real_mul_int").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(5.2 / 2.0),
        find_connstant_value(&index, "real_div_int").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(5.2 % 2.0),
        find_connstant_value(&index, "real_mod_int").unwrap()
    );
    // REAL - REAL
    debug_assert_eq!(
        &create_real_literal(4.4),
        find_connstant_value(&index, "real_plus_real").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(3.3 - 1.1),
        find_connstant_value(&index, "real_minus_real").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(3.3 * 1.1),
        find_connstant_value(&index, "real_mul_real").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(5.3 / 2.1),
        find_connstant_value(&index, "real_div_real").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(5.3 % 2.1),
        find_connstant_value(&index, "real_mod_real").unwrap()
    );
    // BOOL - BOOL
    debug_assert_eq!(
        &create_bool_literal(true),
        find_connstant_value(&index, "bool_and_bool").unwrap()
    );
    debug_assert_eq!(
        &create_bool_literal(true),
        find_connstant_value(&index, "bool_or_bool").unwrap()
    );
    debug_assert_eq!(
        &create_bool_literal(false),
        find_connstant_value(&index, "bool_xor_bool").unwrap()
    );
    debug_assert_eq!(
        &create_bool_literal(false),
        find_connstant_value(&index, "not_bool").unwrap()
    );
}

#[test]
fn const_references_bool_bit_functions_behavior_evaluation() {
    // GIVEN some bit-functions used as initializers
    let (_, index) = parse(
        "VAR_GLOBAL CONSTANT
            _true : BOOL := TRUE;
            _false : BOOL := FALSE;
        END_VAR
        
        VAR_GLOBAL CONSTANT
            a : WORD := _true;
            b : WORD := a AND _false;
            c : WORD := a OR _false;
            d : WORD := a XOR _true;
            e : WORD := NOT a;
        END_VAR
        ",
    );

    // WHEN compile-time evaluation is applied
    let (index, unresolvable) = evaluate_constants(index);

    // THEN everything got resolved
    debug_assert_eq!(EMPTY, unresolvable);
    // AND the index should have literal values
    debug_assert_eq!(
        &create_bool_literal(true),
        find_connstant_value(&index, "a").unwrap()
    );
    debug_assert_eq!(
        &create_bool_literal(false),
        find_connstant_value(&index, "b").unwrap()
    );
    debug_assert_eq!(
        &create_bool_literal(true),
        find_connstant_value(&index, "c").unwrap()
    );
    debug_assert_eq!(
        &create_bool_literal(false),
        find_connstant_value(&index, "d").unwrap()
    );
    debug_assert_eq!(
        &create_bool_literal(false),
        find_connstant_value(&index, "e").unwrap()
    );
}

#[test]
fn const_references_int_bit_functions_behavior_evaluation() {
    // GIVEN some bit-functions used as initializers
    let (_, index) = parse(
        "VAR_GLOBAL CONSTANT
            _0x00ff : WORD := 16#00FF;
        END_VAR
        
        VAR_GLOBAL CONSTANT
            a : WORD := 16#FFAB;
            b : WORD := a AND _0x00ff;
            c : WORD := a OR _0x00ff;
            d : WORD := a XOR _0x00ff;
            e : WORD := NOT a;
        END_VAR
        ",
    );

    // WHEN compile-time evaluation is applied
    let (index, unresolvable) = evaluate_constants(index);

    // THEN everything got resolved
    debug_assert_eq!(EMPTY, unresolvable);
    // AND the index should have literal values
    debug_assert_eq!(
        &create_int_literal(0xFFAB),
        find_connstant_value(&index, "a").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(0x00AB),
        find_connstant_value(&index, "b").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(0xFFFF),
        find_connstant_value(&index, "c").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(0xFF54),
        find_connstant_value(&index, "d").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(0x0054),
        find_connstant_value(&index, "e").unwrap()
    );
}
#[test]
fn illegal_cast_should_not_be_resolved() {
    // GIVEN some bit-functions used as initializers
    let (_, index) = parse(
        "VAR_GLOBAL CONSTANT
            a : INT := BOOL#16#00FF;
        END_VAR
       ",
    );

    // WHEN compile-time evaluation is applied
    let (index, unresolvable) = evaluate_constants(index);

    // THEN a could not be resolved, because the literal is invalid
    debug_assert_eq!(
        vec![UnresolvableConstant::new(
            global!(index, "a"),
            "Cannot resolve constant: BOOL#LiteralInteger { value: 255 }"
        )],
        unresolvable
    );
}

#[test]
fn division_by_0_should_fail() {
    // GIVEN some bit-functions used as initializers
    let (_, index) = parse(
        "VAR_GLOBAL CONSTANT
            zero_int : INT := 0;
            zero_real : REAL := 0.0;

            a : REAL := 5 / zero_int;
            b : REAL := 5 / zero_real;
            c : REAL := 5.0 / zero_int;
            d : REAL := 5.0 / zero_real;
            
            aa : REAL := 5 MOD zero_int;
            bb : REAL := 5 MOD zero_real;
            cc : REAL := 5.0 MOD zero_int;
            dd : REAL := 5.0 MOD zero_real;

        END_VAR
       ",
    );

    // WHEN compile-time evaluation is applied
    let (index, unresolvable) = evaluate_constants(index);
    // THEN division by 0 are reported - note that division by 0.0 results in infinitya
    debug_assert_eq!(
        vec![
            UnresolvableConstant::new(global!(&index, "a"), "Attempt to divide by zero"),
            UnresolvableConstant::new(global!(&index, "c"), "Attempt to divide by zero"),
            UnresolvableConstant::new(
                global!(&index, "aa"),
                "Attempt to calculate the remainder with a divisor of zero"
            ),
            UnresolvableConstant::new(
                global!(&index, "cc"),
                "Attempt to calculate the remainder with a divisor of zero"
            ),
        ],
        unresolvable
    );
    // AND the real divisions are inf or nan
    debug_assert_eq!(
        &create_real_literal(f64::INFINITY),
        find_connstant_value(&index, "b").unwrap()
    );
    debug_assert_eq!(
        &create_real_literal(f64::INFINITY),
        find_connstant_value(&index, "d").unwrap()
    );

    if let AstStatement::LiteralReal { value, .. } = find_connstant_value(&index, "bb").unwrap() {
        assert!(value.parse::<f64>().unwrap().is_nan());
    } else {
        unreachable!()
    }

    if let AstStatement::LiteralReal { value, .. } = find_connstant_value(&index, "dd").unwrap() {
        assert!(value.parse::<f64>().unwrap().is_nan());
    } else {
        unreachable!()
    }
}

#[test]
fn const_references_not_function_with_signed_ints() {
    // GIVEN some bit-functions used as initializers
    let (_, index) = parse(
        "VAR_GLOBAL CONSTANT
            _0x00ff : INT := 16#00FF; //255
        END_VAR
        
        VAR_GLOBAL CONSTANT
            a : INT := INT#16#FFAB;//-85;
            aa : INT := WORD#16#FFAB;//65xxx;
            b : INT := a AND _0x00ff; //171
            c : INT := a OR _0x00ff; //-1
            d : INT := a XOR _0x00ff; //-172
            e : INT := NOT a; //84
        END_VAR
        ",
    );

    // WHEN compile-time evaluation is applied
    let (index, unresolvable) = evaluate_constants(index);

    // THEN everything got resolved
    debug_assert_eq!(EMPTY, unresolvable);
    // AND the index should have literal values
    debug_assert_eq!(
        &create_int_literal(-85),
        find_connstant_value(&index, "a").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(0x0000_ffab),
        find_connstant_value(&index, "aa").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(171),
        find_connstant_value(&index, "b").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(-1),
        find_connstant_value(&index, "c").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(-172),
        find_connstant_value(&index, "d").unwrap()
    );
    debug_assert_eq!(
        &create_int_literal(84),
        find_connstant_value(&index, "e").unwrap()
    );
}

#[test]
fn const_references_to_bool_compile_time_evaluation() {
    // GIVEN some BOOL index used as initializers
    let (_, index) = parse(
        "VAR_GLOBAL CONSTANT
            x : BOOL := TRUE;
            y : BOOL := FALSE;
            z : BOOL := y;
        END_VAR
        
        VAR_GLOBAL CONSTANT
            a : BOOL := x;
            b : BOOL := y OR NOT y;
            c : BOOL := z AND NOT z;
        END_VAR
        ",
    );

    // WHEN compile-time evaluation is applied
    let (index, unresolvable) = evaluate_constants(index);

    // THEN a,b,and c got their correct initial-literals
    debug_assert_eq!(EMPTY, unresolvable);
    debug_assert_eq!(
        find_connstant_value(&index, "a"),
        Some(&create_bool_literal(true))
    );
    debug_assert_eq!(
        find_connstant_value(&index, "b"),
        Some(&create_bool_literal(true))
    );
    debug_assert_eq!(
        find_connstant_value(&index, "c"),
        Some(&create_bool_literal(false))
    );
}

#[test]
fn not_evaluatable_consts_are_reported() {
    // GIVEN some BOOL index used as initializers
    let (_, index) = parse(
        "VAR_GLOBAL CONSTANT
            a : INT := 1;
            b : INT := a;
            c : INT;
            d : INT := c;
        END_VAR",
    );

    // WHEN compile-time evaluation is applied
    let (index, unresolvable) = evaluate_constants(index);

    // THEN d cannot be evaluated, c was not attempted, there is no const-expression
    debug_assert_eq!(
        vec![UnresolvableConstant::incomplete_initialzation(&global!(
            index, "d"
        )),],
        unresolvable
    );
}

#[test]
fn evaluating_constants_can_handle_recursion() {
    // GIVEN some BOOL index used as initializers
    let (_, index) = parse(
        "VAR_GLOBAL CONSTANT
            a : INT := d;
            b : INT := a;
            c : INT := b;
            d : INT := a;

            aa : INT := 4;
            bb : INT := aa;
        END_VAR",
    );

    // WHEN compile-time evaluation is applied
    let (index, unresolvable) = evaluate_constants(index);

    // THEN a,b,c,d could not be resolved (ciruclar dependency)
    debug_assert_eq!(
        vec![
            UnresolvableConstant::incomplete_initialzation(&global!(index, "a")),
            UnresolvableConstant::incomplete_initialzation(&global!(index, "b")),
            UnresolvableConstant::incomplete_initialzation(&global!(index, "c")),
            UnresolvableConstant::incomplete_initialzation(&global!(index, "d")),
        ],
        unresolvable
    );
    // AND aa and bb where resolved correctly
    debug_assert_eq!(
        find_connstant_value(&index, "aa"),
        Some(&create_int_literal(4))
    );
    debug_assert_eq!(
        find_connstant_value(&index, "bb"),
        Some(&create_int_literal(4))
    );
}

#[test]
fn const_string_initializers_should_be_converted() {
    // GIVEN some STRING constants used as initializers
    let (_, index) = parse(
        r#"VAR_GLOBAL CONSTANT
            a : STRING := 'Hello';
            b : WSTRING := "World";
        END_VAR
        
        VAR_GLOBAL CONSTANT
            aa : STRING := b;
            bb : WSTRING := a;
        END_VAR
        "#,
    );

    // WHEN compile-time evaluation is applied
    let (index, unresolvable) = evaluate_constants(index);

    // THEN all should be resolved
    debug_assert_eq!(EMPTY, unresolvable);

    // AND the globals should have gotten their values

    debug_assert_eq!(
        find_connstant_value(&index, "aa"),
        Some(AstStatement::LiteralString {
            value: "World".into(),
            is_wide: false,
            id: 0,
            location: SourceRange::undefined()
        })
    );
    debug_assert_eq!(
        find_connstant_value(&index, "bb"),
        Some(AstStatement::LiteralString {
            value: "Hello".into(),
            is_wide: true,
            id: 0,
            location: SourceRange::undefined()
        })
    );
}
