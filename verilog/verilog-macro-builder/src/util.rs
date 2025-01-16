use sv_parser as sv;

pub fn evaluate_numeric_constant_expression(
    ast: &sv::SyntaxTree,
    expression: &sv::ConstantExpression,
) -> usize {
    match expression {
        sv::ConstantExpression::ConstantPrimary(constant_primary) => {
            match &**constant_primary {
                sv::ConstantPrimary::PrimaryLiteral(primary_literal) => {
                    match &**primary_literal {
                        sv::PrimaryLiteral::Number(number) => match &**number {
                            sv::Number::IntegralNumber(integral_number) => {
                                match &**integral_number {
                                    sv::IntegralNumber::DecimalNumber(
                                        decimal_number,
                                    ) => match &**decimal_number {
                                        sv::DecimalNumber::UnsignedNumber(
                                            unsigned_number,
                                        ) => ast
                                            .get_str_trim(
                                                &unsigned_number.nodes.0,
                                            )
                                            .unwrap()
                                            .parse()
                                            .unwrap(),
                                        sv::DecimalNumber::BaseUnsigned(
                                            decimal_number_base_unsigned,
                                        ) => todo!(),
                                        sv::DecimalNumber::BaseXNumber(
                                            decimal_number_base_xnumber,
                                        ) => todo!(),
                                        sv::DecimalNumber::BaseZNumber(
                                            decimal_number_base_znumber,
                                        ) => todo!(),
                                    },
                                    sv::IntegralNumber::OctalNumber(
                                        octal_number,
                                    ) => todo!(),
                                    sv::IntegralNumber::BinaryNumber(
                                        binary_number,
                                    ) => todo!(),
                                    sv::IntegralNumber::HexNumber(
                                        hex_number,
                                    ) => todo!(),
                                }
                            }
                            sv::Number::RealNumber(real_number) => {
                                panic!("Real number")
                            }
                        },
                        _ => todo!("Other constant primary literals"),
                    }
                }
                _ => panic!("Not a number"),
            }
        }
        sv::ConstantExpression::Unary(constant_expression_unary) => {
            todo!("Constant unary expressions")
        }
        sv::ConstantExpression::Binary(constant_expression_binary) => {
            todo!("Constant binary expressions")
        }
        sv::ConstantExpression::Ternary(constant_expression_ternary) => {
            todo!("Constant ternary expressions")
        }
    }
}
