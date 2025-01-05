module main(
    input single_input,
    input[7:0] small_input,
    input[63:0] medium_input,
    input[127:0] big_input,
    output single_output,
    output[7:0] small_output,
    output[63:0] medium_output,
    output[127:0] big_output
);
    assign single_output = single_input;
    assign small_output = small_input;
    assign medium_output = medium_input;
    assign big_output = big_input;
endmodule
