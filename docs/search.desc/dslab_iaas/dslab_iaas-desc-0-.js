searchState.loadedDescShard("dslab_iaas", 0, "DSLab IaaS\nTrait for implementation of custom components.\nTools for running experiments with multiple simulation …\nThe main entry point for simulation configuration and …\nCommon data structures.\nEnergy meter calculates the host energy consumption.\nStandard simulation events.\nHost manager representing a physical machine.\nResource load models.\nService that provides information about current state of …\nComponent managing the authoritative copy of resource pool …\nResource pool state.\nComponent performing allocation of resources for new VMs.\nService-level agreement violation metrics.\nRepresentations of virtual machine and its status.\nComponent that provides information about all VMs in the …\nVirtual machine placement algorithms.\nDescribes a specific resource allocation, is used to pass …\nDescribes a result of checking the allocation feasibility.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nExperiment configuration.\nConfig utils.\nSimulation configuration.\nRepresents experiment configuration and allows to obtain …\nPrints the current config state with values of dynamic …\nReturns the argument unchanged.\nCreates experiment config by reading parameter values from …\nReturns configuration for next simulation run.\nCalls <code>U::from(self)</code>.\nParses config value string, which consists of two parts - …\nParses options string from config value, returns map with …\nHolds configuration of a single physical host or a set of …\nHolds configuration of a single scheduler or a set of …\nRepresents simulation configuration.\nHolds information about the used VM trace dataset.\nVM placement algorithm used by scheduler(s).\nPeriod is seconds for waiting before retrying failed …\nWhether to schedule VMs based on real resource utilization …\nNumber of such hosts.\nNumber of such schedulers.\nHost CPU capacity.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCreates simulation config by reading parameter values from …\nConfigurations of physical hosts.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nHost memory capacity in GB.\nMessage delay in seconds for communications via network.\nHost name. Should be set if count = 1.\nScheduler name. Should be set if count = 1.\nHost name prefix. Full name is produced by appending host …\nScheduler name prefix. Full name is produced by appending …\nNetwork throughput in GB/s. Currently used to compute VM …\nDataset path.\nConfigurations of VM schedulers.\nPeriod length in seconds for sending statistics from host …\nLength of simulation in seconds (for public datasets only).\nDuration in seconds between simulation steps.\nUsed VM trace dataset.\nDataset type.\nTimeout in seconds after which unallocated VM becomes …\nVM start duration in seconds.\nVM stop duration in seconds.\nEnergy meter structure.\nReturns the total energy consumption.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCreates component.\nInvoked each time the host power consumption is changed to …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nRepresents a single physical machine or host for short, …\nReturns the total amount of allocated vCPUs.\nReturns the current CPU load (used/total) by summing the …\nReturns the host CPU capacity.\nReturns the current power consumption.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the total SLAV value.\nReturns the total energy consumption.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns the total amount of allocated memory.\nReturns the current memory load (used/total) by summing …\nReturns the host RAM capacity.\nThe simplest load model, the constant load.\nA resource load model is a function, which defines load of …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nHost state contains resource capacity and current actual …\nThis component stores the information about current host …\nAdds new host to internal storage.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the state of specified host.\nGet all host states.\nReturns IDs of active VMS on the specified host.\nReturns an iterator of IDs and states of all hosts.\nReturns component ID.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCreates component.\nThis component maintains the authoritative copy of …\nAdds new host to resource pool state.\nRegisters scheduler so that PS can notify it about …\nProcesses direct allocation commit request bypassing the …\nReturns the argument unchanged.\nReturns component ID.\nReturns a copy of the current resource pool state (e.g. to …\nCalls <code>U::from(self)</code>.\nCreates component.\nStores host properties (resource capacity) and state …\nAdds host to resource pool.\nApplies the specified application on the specified host.\nChecks if the specified allocation is currently possible …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns CPU capacity of the specified host currently in …\nReturns memory capacity of the specified host currently in …\nReturns the amount of available vCPUs on the specified …\nReturns the amount of available memory on the specified …\nReturns the CPU allocation rate (ratio of allocated to …\nReturns host info by its ID.\nReturns the number of hosts.\nReturns IDs of all hosts.\nReturns an iterator over all hosts.\nReturns the memory allocation rate (ratio of allocated to …\nReturns the total CPU capacity of the specified host.\nReturns the total memory capacity of the specified host.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCreates host info with specified total and available host …\nCreates empty resource pool state.\nRemoves the specified allocation on the specified host.\nScheduler processes VM allocation requests by selecting …\nAdds host to local resource pool state.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCreates scheduler with specified VM placement algorithm.\nTrait for implementation of host-level SLA violation …\nOverload Time Fraction (OTF) metric.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalled whenever the host’s CPU load is changed to update …\nReturns the current metric value.\nUsed to define resource consumption of virtual machine.\nRepresents virtual machine (VM).\nStatus of virtual machine.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the current CPU load of VM by invoking the CPU …\nReturns the current memory load of VM by invoking the …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns VM lifetime (it is updated when VM is migrated).\nCreates resource consumer with specified parameters.\nCreates virtual machine with specified parameters.\nChanges VM lifetime. It is called only due to VM migration.\nSets VM start time. Can be called multiple times due to VM …\nReturns VM start duration (the value is taken from the …\nReturns VM start time (it is updated when VM is migrated).\nReturns VM stop duration (the value is taken from the …\nCreates resource consumer with specified constant load.\nCreates resource consumer with constant 100% load.\nAPI to access information about virtual machines.\nReturns the argument unchanged.\nGenerates new VM ID if user did not pass any.\nReturns component ID.\nReturns the reference to VM information by VM ID.\nReturns resource allocation for specified VM.\nReturns the total VM count.\nReturns the status of specified VM.\nCalls <code>U::from(self)</code>.\nCreates component.\nProcesses VM status change event emitted by host manager.\nRegisters information about new VM. Called when VM is …\nTrait for implementation of multi-VM placement algorithms.\nTrait for implementation of VM placement algorithms.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nBest Fit algorithm.\nBest Fit with threshold algorithm.\nCosine Similarity algorithm.\nDelta Perp-Distance algorithm.\nDot Product algorithm.\nFirst Fit algorithm.\nL2 Norm Diff algorithm.\nRack anti-affinity algorithm.\nDot Product with resources weights algorithm.\nWorst Fit algorithm.\nUses the most loaded (by allocated CPU) suitable host.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nUses the most loaded (by actual CPU load) suitable host. …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nMaximizes the cosine of the angle between the host’s …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nMaximizes the improvement (absolute decrease) of the …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nMaximizes the dot product between the host’s available …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nUses the first suitable host.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nMinimizes the difference between the VM’s resource usage …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nMulti VM placement algorithm which places each VM from …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nMaximizes the weighted dot product between the host’s …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nUses the least loaded (by allocated CPU) suitable host.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nInitializes component, emits required events.\nCreates new component with provided simulation context.\nImplements execution of experiment.\nTrait for implementing custom callbacks for simulation …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nRuns upon the completion of a simulation run, returns …\nRuns before starting a simulation run.\nRuns on each step of a simulation run, returns false if …\nRuns the experiment using the specified number of threads.\nDataset reader for Azure Trace for Packing 2020.\nTrait for dataset readers.\nVM dataset types.\nDataset reader for Huawei VM Placements Dataset (2021).\nDataset reader for standard JSON format.\nComponent performing automatic migration of VMs.\nDataset reader for Azure Trace for Packing 2020.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCreates dataset reader.\nLoads the dataset from CSV files with VM types and …\nRepresents information about a single virtual machine from …\nReturns the argument unchanged.\nReturns the next VM from dataset (if any).\nCalls <code>U::from(self)</code>.\nHolds supported VM dataset types.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nDataset reader for Huawei VM Placements Dataset.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCreates dataset reader.\nLoads the dataset from the original CSV file.\nDataset reader for standard JSON format.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCreates dataset reader.\nLoads the dataset from JSON file with VM requests.\nThis component performs automatic migration of VMs to …\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nUsed to provide the references to standard components …\nRepresents a simulation, provides methods for its …\nCreates new host with specified name and resource …\nCreates new host with specified name and resource …\nCreates new scheduler with specified name and VM placement …\nReturns the average CPU load across all hosts.\nReturns the average memory load across all hosts.\nSwitches API to batch mode for building multi-VM requests. …\nCreates custom component and adds it to the simulation.\nReturns the main simulation context.\nReturns the current CPU allocation rate (% of overall CPU …\nReturns the current simulation time.\nReturns the total number of created events.\nReturns the argument unchanged.\nReturns the reference to host manager (host energy …\nReturns the reference to host manager (host energy …\nReturns the map with references to host managers.\nCalls <code>U::from(self)</code>.\nReturns the identifier of component by its name.\nReturns the current memory allocation rate (% of overall …\nSends VM migration request to the specified target host.\nReturns the reference to monitoring component (provides …\nCreates a simulation with specified config.\nReturns the reference to scheduler.\nReturns the reference to host scheduler.\nOverrides the used host power model.\nOverrides the used host-level SLAV metric.\nReturns the simulation config.\nSpawns the current batch as a single multi-VM requests and …\nCreates new VM with specified properties, registers it in …\nCreates new VM with specified properties and spawns it on …\nCreates new VM with specified properties, registers it in …\nSpawns all VMs from the given dataset.\nSteps through the simulation with duration limit (see …\nSteps through the simulation until the specified time (see …\nPerforms the specified number of steps through the …\nReturns the reference to VM information.\nReturns the reference to VM API component (provides …\nReturns the ID of host that runs the specified VM.\nReturns the (possibly slightly outdated) status of …\nCreates a simulation with specified config.")