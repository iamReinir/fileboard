document.getElementById('uploadBtn').addEventListener('click', async () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.multiple = true; // allow multiple files
    input.click();

    input.onchange = async () => {
    const files = input.files;
    const formData = new FormData();
    for (let file of files) {
        formData.append('files', file); // API must accept 'files' as an array or handle multiple
    }

    try {
        const response = await fetch('{{host}}', {
        method: 'POST',
        body: formData
        });
        const message = await response.text();
        if (response.ok) {
        alert('Files uploaded successfully!');
        } else {
        alert('Upload failed. ' + message);
        }
        window.location.reload();
    } catch (err) {
        console.error(err);
        alert('Error occurred during upload.');
    }
    };
});

document.getElementById('mkdirBtn').addEventListener('click', async () => {
    let path = window.location.pathname;
    let foldername = prompt("Foldername:");
    try {
        const response = await fetch('{{host}}' + foldername, {
        method: 'PUT'});
        const message = await response.text();
        if (response.ok) {
        alert('Folder created!');
        } else {
        alert('Folder created failed. ' + message);
        }
        window.location.reload();
    } catch (err) {
        console.error(err);
        alert('Error occurred during folder creation.');
    }
});

async function move_req(event, path, filename)
{
    event.preventDefault();
    const newPath = prompt("Move or rename file:", path + "/" + filename);
    if(newPath == null)
        return;
    try {
        const res = await fetch("{{host}}" + filename, {
            method: "PATCH",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({
                destination: newPath
            })
        });
        if (!res.ok) {
            const error = await res.text();
            alert("Error: " + error);
            return;
        }
        alert("Moved!");
        location.reload(); // or refresh part of the page
    } catch (err) {
        alert("Failed to move file: " + err.message);
    }
}

async function del_req(event, filename)
{
    event.preventDefault();
    if(!confirm('Do you really want to move this file to trash?:\n' + filename)){
        return;
    }
    try {
        const res = await fetch("{{host}}" + filename, {
            method: "DELETE"
        });
        if (!res.ok) {
            const error = await res.text();
            alert("Error: " + error);
            return;
        }
        alert("Moved to trash!");
        location.reload(); // or refresh part of the page
    } catch (err) {
        alert("Failed to trash file: " + err.message);
    }
}