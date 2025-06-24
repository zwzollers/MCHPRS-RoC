module command_controller #(
    parameter ROC_INPUTS,
    parameter ROC_OUTPUTS,
    parameter ROC_OUTPUT_BYTES
) (
    input                       i_clk,
    input                       i_rx,
    input   [ROC_OUTPUTS-1:0]   i_roc_outputs,
    output  [ROC_INPUTS-1:0]    o_roc_inputs,
    output  reg                 o_tx,
    output  [31:0]              o_roc_tps,
    output  reg                 o_roc_en
);
    /*---------------------------------
                PARAMETERS
    ---------------------------------*/
    parameter
        BYTES_PER_COMMANDS      = 6;
    /*---------------------------------
                  COMMANDS
    ---------------------------------*/
    parameter 
        CMD_RESET               = 8'hC0,
        CMD_PING                = 8'hC1,
        CMD_GET_OUTPUTS         = 8'hC2, 
        CMD_CAPTURE             = 8'hC3,
        CMD_SET_INPUT           = 8'hC4,
        CMD_SET_RTPS            = 8'hC5, 
        CMD_LOAD_ROM            = 8'hC6, 
        CMD_DEBUG_LED           = 8'hC7,
        CMD_FAIL_ACK            = 8'hC8,
        EOC                     = 8'hA5;

    /*---------------------------------
              SERIAL RECEIVER
    ---------------------------------*/
    wire            rx_new;
    wire[7:0]       rx_data;

    uart_rx #(
        .BAUD_DIVIDER_COUNT(20)
    ) rx (
        .i_clk(i_clk),
        .i_rx(i_rx),

        .o_new_data(rx_new),
        .o_data(rx_data)
    );

    /*---------------------------------
             SERIAL TRANSMITTER
    ---------------------------------*/
    reg             r_tx_start  = 1'b0;
    reg[7:0]        r_tx_data   = 8'd0;
    wire            tx_done;
    
    uart_tx #(
        .BAUD_DIVIDER_COUNT(20)
    ) tx (
        .i_clk(i_clk), 
        .i_start(r_tx_start), 
        .i_data(r_tx_data), 

        .o_done(tx_done), 
        .o_tx(o_tx)
    );

    /*---------------------------------
            COMMAND STATE MACHINE
    ---------------------------------*/
    parameter
        s_IDLE                  = 3'b000,
        s_RX                    = 3'b001,
        s_RX_WAIT               = 3'b010,
        s_CMD_CHECK             = 3'b011,
        s_CMD_PROCESS           = 3'b100,
        s_TX_WAIT               = 3'b101,
        s_END                   = 3'b110,
        s_FAILSAFE              = 3'b111;
    reg[2:0]        r_state     = s_IDLE;

    reg[BYTES_PER_COMMANDS-1:0] [7:0] r_cmd;
    reg[2:0]        r_cmd_i     = 3'd0;

    wire[23:0]      three_byte;
    assign          three_byte = {r_cmd[1], r_cmd[2], r_cmd[3]}; 

    wire[31:0]      four_byte;
    assign          four_byte = {r_cmd[1], r_cmd[2], r_cmd[3], r_cmd[4]}; 

    reg[(ROC_OUTPUT_BYTES*8)-1:0]    r_roc_outputs;

    reg[ROC_INPUTS-1:0]     r_roc_inputs;
    assign          o_roc_inputs = r_roc_inputs;

    reg[31:0]       r_tps       = 32'd0;
    assign          o_roc_tps   = r_tps;

    reg[23:0]       r_output_i  = 24'd0;

    always @(posedge i_clk) begin
        case (r_state)
            s_IDLE        : begin
                r_tx_start              <= 1'b0;
                r_output_i              <= 24'd0;
                r_cmd_i                 <= 3'd0;
                if (rx_new)
                    r_state             <= s_RX;
            end
            s_RX          : begin
                r_cmd[r_cmd_i]          <= rx_data;
                r_cmd_i                 <= r_cmd_i + 1;
                r_tx_data               <= rx_data;
                r_tx_start              <= 1'b1;

                if (r_cmd_i >= BYTES_PER_COMMANDS-1) 
                    r_state             <= s_CMD_CHECK;
                else
                    r_state             <= s_RX_WAIT;
            end

            s_RX_WAIT     : begin
                if (rx_new)
                    r_state             <= s_RX;
                    r_tx_start          <= 1'b0;
            end

            s_CMD_CHECK   : begin
                if (r_cmd[BYTES_PER_COMMANDS-1] == EOC && 
                        r_cmd[0] >= CMD_RESET && 
                        r_cmd[0] <= CMD_FAIL_ACK)
                    r_state             <= s_TX_WAIT;
                else 
                    r_state             <= s_FAILSAFE;
            end

            s_CMD_PROCESS : begin
                case (r_cmd[0])
                    CMD_RESET       : begin
                        r_state         <= s_END;
                    end
                    CMD_PING        : begin
                        r_state         <= s_END;
                    end 
                    CMD_GET_OUTPUTS : begin
                        if (r_output_i >= ROC_OUTPUT_BYTES) begin
                            r_state     <= s_END;
                        end
                        else begin
                            r_tx_data   <= r_roc_outputs[7:0];
                            r_tx_start  <= 1'b1;
                            r_roc_outputs <= r_roc_outputs >> 8;
                            r_output_i  <= r_output_i + 1;
                            r_state     <= s_TX_WAIT;
                        end
                    end 
                    CMD_CAPTURE     : begin
                        r_roc_outputs   <= i_roc_outputs;
                        r_state         <= s_END;
                    end
                    CMD_SET_INPUT   : begin
                        r_roc_inputs[three_byte] <= r_cmd[4][0];
                        r_state         <= s_END;
                    end
                    CMD_SET_RTPS    : begin
                        r_tps           <= four_byte;
                        r_state         <= s_END;
                    end 
                    CMD_LOAD_ROM    : begin

                    end 
                    CMD_FAIL_ACK    : begin

                    end
                    default         : begin
                        r_state         <= s_FAILSAFE;
                    end
                endcase
            end

            s_TX_WAIT     : begin
                r_tx_start              <= 1'b0;
                if (tx_done) 
                    r_state             <= s_CMD_PROCESS;
            end

            s_END         : begin
                r_tx_data               <= 8'd0;
                r_output_i              <= 24'd0;
                r_tx_start              <= 1'b0;
                r_cmd_i                 <= 3'd0;
                r_state                 <= s_IDLE;
            end

            s_FAILSAFE    : begin
                if (tx_done) begin
                    r_tx_start          <= 1'b1;
                    r_tx_data           <= 8'h5A;
                end
                else begin
                    r_tx_start          <= 1'b0;
                end
            end

            default       : begin
                r_state                 <= s_FAILSAFE;
            end
        endcase
    end
endmodule