import os
import subprocess
import re 


directory = os.path.dirname(os.path.realpath(__file__))

os.chdir(directory)

for filename in os.listdir(directory):
  if filename.endswith(".frag") or filename.endswith(".comp") or filename.endswith(".vert"): 
    symbolName = filename.split(".")[0] + "_" + filename.split(".")[1]
    spirName = "_" + symbolName + ".spv"
    subprocess.run(["C:\\VulkanSDK\\1.3.216.0\\Bin\\glslc.exe", "-g", "-O", filename,  "-o", spirName])
    # "" -g -O $1 -o $1.spv
    continue
