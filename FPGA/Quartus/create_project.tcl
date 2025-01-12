package require ::quartus::project

# Create project
project_new -revision FPGA-MCHPRS FPGA-MCHPRS

# Assignments
set_global_assignment -name FAMILY "Cyclone V"
set_global_assignment -name DEVICE 5CSEMA5F31A7
set_global_assignment -name TOP_LEVEL_ENTITY top
set_global_assignment -name ORIGINAL_QUARTUS_VERSION 23.1STD.1


#set_global_assignment -name OPTIMIZATION_TECHNIQUE AREA
#set_global_assignment -name PHYSICAL_SYNTHESIS_COMBO_LOGIC ON
#set_global_assignment -name ROUTER_LCELL_INSERTION_AND_LOGIC_DUPLICATION ON
#set_global_assignment -name ROUTER_TIMING_OPTIMIZATION_LEVEL MAXIMUM


set_global_assignment -name VERILOG_FILE Verilog/uart.v

set_global_assignment -name VERILOG_FILE Verilog/redstone.v
set_global_assignment -name VERILOG_FILE Verilog/components.v
set_global_assignment -name VERILOG_FILE Verilog/top.v
set_global_assignment -name VERILOG_FILE Verilog/clk_div.v

set_global_assignment -name SOURCE_FILE Verilog/Tick/Tick.cmp
set_global_assignment -name QIP_FILE Verilog/Tick/Tick.qip
set_global_assignment -name SIP_FILE Verilog/Tick/Tick.sip
set_instance_assignment -name GLOBAL_SIGNAL GLOBAL_CLOCK -to "Tick:tick_clk|outclk_0"

set_location_assignment PIN_AJ17 -to i_RX
set_location_assignment PIN_AJ16 -to o_TX
set_location_assignment PIN_AA14 -to i_buttons[0]

# Commit assignments
export_assignments

# Close project
project_close