const fs = require('fs');
const path = require('path');

const targetFile = path.join(__dirname, '..', 'node_modules', '@hyperledger', 'caliper-core', 'lib', 'manager', 'monitors', 'monitor-docker.js');

if (!fs.existsSync(targetFile)) {
    console.error(`[Patch] Could not find ${targetFile}. Make sure you have run 'npm install' first.`);
    process.exit(1);
}

let content = fs.readFileSync(targetFile, 'utf8');

let patched = false;

// 1. Fix memory inactive_file reference (Podman doesn't always provide stats.inactive_file)
if (content.includes('stat.memory_stats.stats.inactive_file')) {
    content = content.replace(
        /const actualMemUsage = stat\.memory_stats\.usage - stat\.memory_stats\.stats\.inactive_file;/g,
        'const actualMemUsage = stat.memory_stats.usage - (stat.memory_stats.stats && stat.memory_stats.stats.inactive_file ? stat.memory_stats.stats.inactive_file : 0);'
    );
    patched = true;
}

// 2. Fix mem_limit reference (Podman uses memory_stats.limit instead of mem_limit in some API versions)
if (content.includes('stat.mem_limit')) {
    content = content.replace(
        /stat\.mem_limit/g,
        'stat.memory_stats.limit'
    );
    patched = true;
}

// 3. Fix CPU delta calculations (Podman lacks reliable precpu_stats)
const originalCpuTotal = /let cpuDelta = stat\.cpu_stats\.cpu_usage\.total_usage - stat\.precpu_stats\.cpu_usage\.total_usage;/;
if (originalCpuTotal.test(content)) {
    const replacementCpuTotal = `if (!this.last_cpu_stats) this.last_cpu_stats = {};
                    let precpu_sys = this.last_cpu_stats[id] ? this.last_cpu_stats[id].system_cpu_usage : (stat.precpu_stats && stat.precpu_stats.system_cpu_usage ? stat.precpu_stats.system_cpu_usage : 0);
                    let precpu_tot = this.last_cpu_stats[id] ? this.last_cpu_stats[id].cpu_usage.total_usage : (stat.precpu_stats && stat.precpu_stats.cpu_usage ? stat.precpu_stats.cpu_usage.total_usage : 0);
                    this.last_cpu_stats[id] = stat.cpu_stats;
                    let cpuDelta = stat.cpu_stats.cpu_usage.total_usage - precpu_tot;`;

    content = content.replace(originalCpuTotal, replacementCpuTotal);

    content = content.replace(
        /let sysDelta = stat\.cpu_stats\.system_cpu_usage - stat\.precpu_stats\.system_cpu_usage;/,
        `let sysDelta = stat.cpu_stats.system_cpu_usage - precpu_sys;`
    );
    patched = true;
}

if (patched) {
    fs.writeFileSync(targetFile, content, 'utf8');
    console.log('[Patch] Successfully patched @hyperledger/caliper-core docker monitor for Podman compatibility.');
} else {
    console.log('[Patch] File is already patched or patterns did not match.');
}
