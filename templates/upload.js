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
