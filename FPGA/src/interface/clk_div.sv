module clk_div #(
    parameter SIZE,
    parameter COUNT
)( 
    input           clk_in, 
    output  wire    clk_out
);

    reg[SIZE-1:0]   r_count     = 0;
    reg             r_clk       = 1'b0;


    always @(posedge clk_in) begin
        if(r_count >= COUNT-1) begin
            r_clk               <= ~r_clk;
            r_count             <= 0;
        end
        else
            r_count             <= r_count + 1;
    end

    assign clk_out  = r_clk;

endmodule  
        