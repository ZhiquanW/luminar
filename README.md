# laminar

# Notes
1. brief design
   1. method calls at inital 
      1. read configuration files
      2. create user information structure based on configuration
   2. method calls at each time step
      1. update system resource information: total cpu usage, total(free) memory usage ...
      2. update users resources allocation
         1. each user: cpu/gpu usage, cpu/gpu memory usage
      3. perform mamagement actions based on the user configuration: kill process ...