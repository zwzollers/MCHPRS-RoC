module tps_clk_div #(
    parameter REF_CLK_SIZE
) (
    input                       i_clk,
    input   [REF_CLK_SIZE-1:0]  i_tps,
    input                       i_en,
    output  reg                 o_clk
);

    reg[REF_CLK_SIZE-1:0]   r_sum;
    reg[REF_CLK_SIZE-1:0]   r_carry;

    wire[REF_CLK_SIZE-1:0]  first_xor;
    assign                  first_xor = r_sum ^ i_tps;    

    always @(posedge i_clk) begin
        o_clk               <= r_sum[REF_CLK_SIZE-1];
        r_sum               <= r_sum ^ r_carry ^ i_tps;
        r_carry             <= ((r_sum & i_tps) | (first_xor & r_carry)) << 1;
    end

endmodule