# Substrato 1046.3.1 — CELLULAR-CHECKPOINT-FPGA Synthesis Script
# Target: Xilinx Alveo U280 (xcu280-fsvh2892-2L-e)
# Integration: 989.y.6.1-FULL (DKES-NTT + Alveo U280 @ 321MHz)
# Arquiteto: ORCID 0009-0005-2697-4668
# Seal: CHECKPOINT-FPGA-1046.3.1-2026-06-03

set project_name "cellular_checkpoint_fpga"
set part_name "xcu280-fsvh2892-2L-e"
set output_dir "./output"

# Create project
create_project $project_name $output_dir -part $part_name -force

# Add source files
add_files -fileset sources_1 {
    ../cellular_checkpoint_rtl_1046.3.v
    ./cellular_checkpoint_wrapper_1046.3.1.v
}

# Add constraints
add_files -fileset constrs_1 ./alveo_u280_constraints_1046.3.1.xdc

# Set top module
set_property top cellular_checkpoint_wrapper [get_filesets sources_1]

# Synthesis settings
set_property STEPS.SYNTH_DESIGN.ARGS.FLATTEN_HIERARCHY none [get_runs synth_1]
set_property STEPS.SYNTH_DESIGN.ARGS.KEEP_EQUIVALENT_REGISTERS true [get_runs synth_1]

# Implementation strategy
set_property strategy Performance_ExplorePostRoutePhysOpt [get_runs impl_1]

# Clock constraint: 321 MHz (3.115 ns period) — matching 989.y.6.1-FULL
set_property -name {STEPS.OPT_DESIGN.ARGS.DIRECTIVE} -value {Explore} -objects [get_runs impl_1]

# Launch synthesis
launch_runs synth_1 -jobs 8
wait_on_run synth_1

# Launch implementation
launch_runs impl_1 -to_step write_bitstream -jobs 8
wait_on_run impl_1

# Report utilization and timing
open_run impl_1
report_utilization -file $output_dir/utilization_report.txt
report_timing_summary -file $output_dir/timing_report.txt
report_power -file $output_dir/power_report.txt

puts "Synthesis complete. Reports in $output_dir"
puts "Target: Alveo U280, 321 MHz, ~7.7us latency"