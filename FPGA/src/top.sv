module top (
	input       i_clk,
	input 	    i_RX,
    input       i_rst,   
	output 		o_TX,
    output      o_tick,
    output      o_debug
);

    /*---------------------------------
                PARAMETERS
    ---------------------------------*/
    `include "parameters.vh"

    /*---------------------------------
             COMMAND CONTROLLER
    ---------------------------------*/
    wire[ROC_INPUTS-1:0]    roc_inputs;
    wire[ROC_OUTPUTS-1:0]   roc_outputs;

    wire[31:0]              roc_tps;
    wire                    roc_clk_en;

    command_controller #(
        .ROC_INPUTS(ROC_INPUTS),
        .ROC_OUTPUTS(ROC_OUTPUTS),
        .ROC_OUTPUT_BYTES((ROC_OUTPUTS+7)>>3)
    ) cmd_ctrl (
        .i_clk(i_clk),
        .i_rx(i_RX),
        .i_roc_outputs(roc_outputs),

        .o_roc_inputs(roc_inputs),
        .o_tx(o_TX),
        .o_roc_tps(roc_tps),
        .o_roc_en(roc_clk_en)
    );

    /*---------------------------------
                TPS DIVIDER
    ---------------------------------*/
    wire                    roc_tps_clk;
    wire                    tick_clk;

    assign o_tick = roc_tps_clk;

    tick_clk (
		.refclk(i_clk),
		.outclk_0(tick_clk),
	);

    tps_clk_div #(
        .REF_CLK_SIZE(29)
    ) (
        .i_clk(tick_clk),
        .i_tps(roc_tps),
        .i_en(roc_clk_en),

        .o_clk(roc_tps_clk)
    );

    /*---------------------------------
           REDSTONE IMPLEMENTATION
    ---------------------------------*/
    RoC #(
        .OUTPUTS(ROC_OUTPUTS),
        .INPUTS(ROC_INPUTS)
    ) roc (
        .tick(roc_tps_clk),
        .inputs(roc_inputs),

        .outputs(roc_outputs)
    );
    
endmodule