module RoC #(
        parameter OUTPUTS,
        parameter INPUTS
    ) (
        input                   tick,
        input   [INPUTS-1:0]    inputs,
        output  [OUTPUTS-1:0]   outputs
    );

endmodule