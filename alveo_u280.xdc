# Substrato 1046.3.1 — Alveo U280 Constraints
# Target: xcu280-fsvh2892-2L-e
# Clock: 321 MHz (period 3.115 ns)
# Arquiteto: ORCID 0009-0005-2697-4668

# === Clock ===
create_clock -name clk -period 3.115 [get_ports clk]
set_property CLOCK_DEDICATED_ROUTE FALSE [get_nets clk]

# === Reset ===
set_property IOSTANDARD LVCMOS18 [get_ports rst_n]
set_property PACKAGE_PIN BC10 [get_ports rst_n]  # Alveo U280 reset pin

# === Theosis Level (16-bit) ===
set_property IOSTANDARD LVCMOS18 [get_ports theosis_level[*]]
for {set i 0} {$i < 16} {incr i} {
    set_property PACKAGE_PIN [format "BD%d" [expr 10 + $i]] [get_ports theosis_level[$i]]
}

# === Repair Done ===
set_property IOSTANDARD LVCMOS18 [get_ports repair_done]
set_property PACKAGE_PIN BE10 [get_ports repair_done]

# === Outputs ===
set_property IOSTANDARD LVCMOS18 [get_ports clock_gated]
set_property PACKAGE_PIN BF10 [get_ports clock_gated]

set_property IOSTANDARD LVCMOS18 [get_ports g0_state]
set_property PACKAGE_PIN BG10 [get_ports g0_state]

set_property IOSTANDARD LVCMOS18 [get_ports checkpoint_phase[*]]
for {set i 0} {$i < 4} {incr i} {
    set_property PACKAGE_PIN [format "BH%d" [expr 10 + $i]] [get_ports checkpoint_phase[$i]]
}

# === Timing Constraints ===
set_input_delay -clock clk -max 0.5 [get_ports rst_n]
set_input_delay -clock clk -max 0.5 [get_ports theosis_level[*]]
set_input_delay -clock clk -max 0.5 [get_ports repair_done]

set_output_delay -clock clk -max 0.5 [get_ports clock_gated]
set_output_delay -clock clk -max 0.5 [get_ports g0_state]
set_output_delay -clock clk -max 0.5 [get_ports checkpoint_phase[*]]

# === False Paths ===
set_false_path -from [get_ports rst_n]

# === Power ===
set_property CONFIG_VOLTAGE 1.8 [current_design]
set_property CFGBVS GND [current_design]