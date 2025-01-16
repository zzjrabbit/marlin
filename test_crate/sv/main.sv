module main(
    input single_input,
    input[31:0] medium_input,
    output single_output,
    output[31:0] medium_output
);
    assign single_output = single_input;
    assign medium_output = medium_input;
endmodule
