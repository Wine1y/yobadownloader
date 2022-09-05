var processNameElement = document.getElementById("progressProcess")
var progressPercentElement = document.getElementById("progressPercent")
var progressBar = document.getElementById("progressBar")

function startDownloading(){
    return invoke(
        'start_downloading',
        {
            "stream": window.loadedFmts[window.currentFmt],
            "timeLine": [Math.round(window.start_seconds), Math.round(window.end_seconds), Math.round(window.videoDuration)],
            "savePath": window.savePath,
            "audioStream": window.need_audio ? window.loadedFmts[0]: null
        }
    )
}

function updateProgress(currentValue, maxValue, processName){
    if(processName != null){
        processNameElement.innerHTML = processName
    }
    if(currentValue != null && maxValue != null){
        let percent = Math.round(currentValue/(maxValue/100))
        progressPercentElement.innerHTML = `${Math.min(percent, 100)}%`
        if(percent >= 90){
            percent+=10
        }
        progressBar.style.background = `linear-gradient(90deg, rgb(249, 253, 15), #cf0268, var(--element-bg) ${Math.min(percent, 110)}%)`
    }
}

function deserializeProgress(progress){
    switch(progress){
        case "Idle":
            updateBreakButton("Back")
            return [[0,100], "Waiting..."]
        case "Done":
            updateBreakButton("Home")
            return [[100,100], "Downloading finished!"]
        case "Canceled":
            updateBreakButton("Back")
            return [[0,100], "Downloading canceled."]
        case "Failed":
            updateBreakButton("Retry")
            return [[0,100], "Downloading failed."]
    }

    let keys = Object.keys(progress)
    if(keys == null || keys.length < 1){return null}
    updateBreakButton("Cancel")
    switch (keys[0]){
        case "DownloadingFFMpeg":
            downloading_progress = progress["DownloadingFFMpeg"]
            return [[downloading_progress[0], downloading_progress[1]], "Downloading FFMpeg..."]
        case "DownloadingVideo":
            downloading_progress = progress["DownloadingVideo"]
            return [[downloading_progress[0], downloading_progress[1]], "Downloading Video..."]
        case "DownloadingAudio":
            downloading_progress = progress["DownloadingAudio"]
            return [[downloading_progress[0], downloading_progress[1]], "Downloading Audio..."]
        case "EncodingAudio":
            ffmpeg_progress = progress["EncodingAudio"]
            return [[ffmpeg_progress[0], ffmpeg_progress[1]], "Converting audio..."]
        case "MergingStreams":
            ffmpeg_progress = progress["MergingStreams"]
            return [[ffmpeg_progress[0], ffmpeg_progress[1]], "Merging streams..."]
        case "CuttingStream":
            ffmpeg_progress = progress["CuttingStream"]
            return [[ffmpeg_progress[0], ffmpeg_progress[1]], "Cutting video..."]
    }
    return null
}

function waitForDownload(){
    startDownloading()
    .then((downloading_result) => {
        requestUserAttention()
        clearInterval(window.checkInterval)
        getDownloadingProgress()
        .then((progress) => {
            result = deserializeProgress(progress)
            if(result != null){
                updateProgress(result[0][0], result[0][1], result[1])
            }
        })
        .catch(error => {
        	clearInterval(window.checkInterval)
            showError(error)
        })
    })
    .catch(error => {
        requestUserAttention()
        showError(error)
    })
}

function updateBreakButton(buttonType){
    link = document.getElementById("breakLink")
    button = document.getElementById("breakButton")
    switch (buttonType) {
        case "Cancel":
            link.href = "#"
            button.innerHTML = "Cancel"
            button.onclick = function(){
                cancelDownloading()
                .catch(error => {
                    showError(error)
                })
                .then(_ =>{
                    updateBreakButton("Back")
                })
            }
            break;
        case "Back":
            link.href = "options.html"
            button.innerHTML = "Back"
            button.onclick = function(){
            	clearInterval(window.checkInterval)
            }
            break;
        case "Home":
            link.href = "index.html"
            button.innerHTML = "Home"
            button.onclick = function(){
            	clearInterval(window.checkInterval)
            }
            break;
        case "Retry":
            link.href = "#"
            button.innerHTML = "Retry"
            button.onclick = function(){
                waitForDownload()
                updateBreakButton("Cancel")
            }
            break;
    }
}
window.checkInterval = setInterval(function(){
    getDownloadingProgress()
        .then(progress => {
            result = deserializeProgress(progress)
            if(result != null){
                updateProgress(result[0][0], result[0][1], result[1])
            }
        })
        .catch(error => {
            showError(error)
        })
}, 500)

updateBreakButton("Back")
waitForDownload()