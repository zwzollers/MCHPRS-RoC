module uart_rx #(
    parameter BAUD_DIVIDER_COUNT
)(
    input                i_clk,
    input                i_rx,
    output   wire        o_new_data,
    output   wire[7:0]   o_data
);
    localparam   
        s_IDLE      = 3'b000,
        s_START     = 3'b001,
        s_DATA      = 3'b010,
        s_STOP      = 3'b011,
        s_CLEANUP   = 3'b100;
    reg [2:0]   r_state     = s_IDLE;

    reg         r_rx_in     = 1'b1;
    reg         r_rx        = 1'b1;

    reg [7:0]   r_clk_cnt   = 0;
    reg [2:0]   r_bit       = 0;
    reg [7:0]   r_data      = 0;
    reg         r_new_data  = 0;
    

    // Purpose: Double-register the incoming data.
    // This allows it to be used in the UART RX Clock Domain.
    // (It removes problems caused by metastability)
    always @(posedge i_clk) begin
        r_rx_in <= i_rx;
        r_rx    <= r_rx_in;
    end


    // Purpose: Control RX state machine
    always @(posedge i_clk) begin
        case (r_state)
            s_IDLE : begin
                r_new_data          <= 1'b0;
                r_clk_cnt           <= 0;
                r_bit               <= 0;

                if (r_rx == 1'b0)
                    r_state         <= s_START;
                else
                    r_state         <= s_IDLE;
            end

            // Check middle of start bit to make sure it's still low
            s_START : begin
                if (r_clk_cnt == (BAUD_DIVIDER_COUNT-1)/2) begin
                    if (r_rx == 1'b0) begin
                        r_clk_cnt   <= 0;
                        r_state     <= s_DATA;
                    end
                    else
                        r_state     <= s_IDLE;
                end
                else begin
                    r_clk_cnt       <= r_clk_cnt + 1;
                    r_state         <= s_START;
                end
            end


            // Wait BAUD_DIVIDER_COUNT-1 clock cycles to sample serial data
            s_DATA : begin
                if (r_clk_cnt < BAUD_DIVIDER_COUNT-1) begin
                    r_clk_cnt       <= r_clk_cnt + 1;
                    r_state         <= s_DATA;
                end
                else begin
                    r_clk_cnt       <= 0;
                    r_data[r_bit]   <= r_rx;

                    // Check if we have received all bits
                    if (r_bit < 7) begin
                        r_bit       <= r_bit + 1;
                        r_state     <= s_DATA;
                    end
                    else begin
                        r_bit       <= 0;
                        r_state     <= s_STOP;                         
                    end
                end
            end
                
            s_STOP : begin
                // Wait BAUD_DIVIDER_COUNT-1 clock cycles for Stop bit to finish
                if (r_clk_cnt < BAUD_DIVIDER_COUNT-1) begin
                    r_clk_cnt       <= r_clk_cnt + 1;
                    r_state         <= s_STOP;                   
                end 
                else begin
                    r_new_data      <= 1'b1;
                    r_clk_cnt       <= 0;
                    r_state         <= s_CLEANUP;
                end
            end           

            s_CLEANUP : begin
                r_state             <= s_IDLE;
                r_new_data          <= 1'b0;
            end

            default :
                r_state             <= s_IDLE;

        endcase
    end   

    assign o_new_data   = r_new_data;
    assign o_data       = r_data;

endmodule

module uart_tx #(
    parameter BAUD_DIVIDER_COUNT
)(
    input               i_clk,
    input               i_start,
    input   wire[7:0]   i_data,
    output  wire        o_done, 
    output  wire        o_tx
);
    parameter   s_IDLE      = 2'b00;
    parameter   s_START     = 2'b01;
    parameter   s_DATA      = 2'b10;
    parameter   s_STOP      = 2'b11;
    reg [1:0]   r_state     = s_IDLE;

    reg         r_tx        = 1'b1;
    reg         r_start     = 1'b0;
    reg         r_prev_start= 1'b0;
    reg         r_started   = 1'b0;

    reg [2:0]   r_bit       = 0;
    reg [7:0]   r_data      = 0;

    reg         r_done      = 0;
    reg         r_prev_done = 0;

    wire baud_clk;
	clk_div #(.SIZE(4), .COUNT(10)) baud_clk_div (.clk_in(i_clk), .clk_out(baud_clk));

    always @(posedge i_clk) begin
        if (~r_prev_start & i_start & ~r_started) begin
            r_start                 <= 1'b1;
            r_data                  <= i_data;
        end
        else if (r_started) begin
            r_start                 <= 1'b0;
        end
        r_prev_start                <= i_start;
    end

    always @(posedge i_clk) begin
        r_prev_done                 <= r_done;
    end

    always @(posedge baud_clk) begin
        case (r_state)
            s_IDLE : begin
                r_done              <= 1'b0;
                r_bit               <= 0;

                if (r_start == 1'b1) begin
                    r_state         <= s_START;
                    r_started       <= 1'b1;
                end
                else
                    r_state         <= s_IDLE;
            end

            s_START : begin
                r_tx                <= 1'b0;
                r_started           <= 1'b0;
                r_state             <= s_DATA;
            end

            s_DATA : begin
                r_tx                <= r_data[r_bit];
                r_started           <= 1'b0;
                
                if (r_bit == 3'd7) begin
                    r_state         <= s_STOP;
                end
                else begin
                    r_bit           <= r_bit + 1;
                end
            end
                
            s_STOP : begin
                r_tx                <= 1'b1;
                r_bit               <= 0;
                r_done              <= 1'b1;
                r_state             <= s_IDLE;
            end           

            default :
                r_state             <= s_IDLE;

        endcase
    end   

    assign o_tx     = r_tx;
    assign o_done   = ~r_prev_done & r_done;

endmodule