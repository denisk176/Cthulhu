async function abortJob(job) {
    await fetch("/port/" + job + "/abort");
}

var reloaders = [];
function createReloader(divId, page) {
    async function reloadHeader() {
        const response = await fetch(page);
        const data = await response.text();
        document.getElementById(divId).innerHTML = data;
    }

    reloaders.push(setInterval(reloadHeader, 1000));
}

createReloader("portstatus", "portstatus.html");

function stopReloaders() {
    for (const r of reloaders) {
        console.log("Stopping reloader: " + r);
        clearInterval(r);
    }
    reloaders = [];
}
