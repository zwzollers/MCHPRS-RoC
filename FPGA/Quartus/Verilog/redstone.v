module redstone (tick, inputs, outputs);
        	input tick;
        	input [num_inputs-1:0] inputs;
        	output [num_outputs:0] outputs;

        
    parameter num_outputs = 1, num_inputs = 1;

	wire w138_8_153;
	assign w138_8_153 = inputs[0];
	wire w140_8_153;
	assign w140_8_153 = inputs[1];
	wire w138_8_156;
	torch #(1'b0) c138_8_156 (.i_clk(tick), .i_in(w139_9_155|w138_9_154), .o_out(w138_8_156));
	wire w139_8_156;
	assign outputs[0] = (w138_8_156|w140_8_156);
	wire w140_8_156;
	torch #(1'b0) c140_8_156 (.i_clk(tick), .i_in(w139_9_155|w140_9_154), .o_out(w140_8_156));
	wire w138_9_154;
	torch #(1'b1) c138_9_154 (.i_clk(tick), .i_in(w138_8_153), .o_out(w138_9_154));
	wire w140_9_154;
	torch #(1'b1) c140_9_154 (.i_clk(tick), .i_in(w140_8_153), .o_out(w140_9_154));
	wire w139_9_155;
	torch #(1'b0) c139_9_155 (.i_clk(tick), .i_in(w138_9_154|w140_9_154), .o_out(w139_9_155));
endmodule