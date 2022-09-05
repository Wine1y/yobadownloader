var invoke = window.__TAURI__.invoke
var webWindow = window.__TAURI__.window.getCurrent()

function checkVideoId(link){
    return invoke('try_get_video_id', {'link': link})
}
 
function setVideoData(video_id){
    return invoke('load_new_video', {'videoId': video_id,})
}
 
function getCurrentVideo(){
    return invoke('get_current_video', {})
}

function getDefaultPath(){
    return invoke('get_default_path', {})
}

function isFFmpegInstalled(){
    return invoke('ffmpeg_installed', {})
}

function downloadFFMpeg(){
    return invoke('download_ffmpeg', {})
}

function getDownloadingProgress(){
    return invoke('get_downloading_progress', {})
}

function cancelDownloading(){
    return invoke('cancel_downloading', {})
}

function confirmPopup(title, text, confirm_text){
    prompt = document.getElementById("confirmPrompt")
    prompt_content = prompt.getElementsByClassName("prompt_content")[0]
    spans = prompt.getElementsByTagName("span")
    buttons = prompt.getElementsByTagName("button")
    spans[0].innerHTML = title
    spans[1].innerHTML = text
    buttons[0].innerHTML = confirm_text

    document.getElementById("confirmPrompt").classList.remove("hidden")
    document.getElementsByClassName("transition-up-down")[0].classList.add("blured")

    function closePopup(){
        document.getElementById("confirmPrompt").classList.add("hidden")
        document.getElementsByClassName("transition-up-down")[0].classList.remove("blured")
    }

    return new Promise(function(resolve, reject){
        buttons[0].onclick = function(){
            closePopup()
            resolve(true)
        }
        window.onclick = function(e){
            if(e.target != prompt_content && !Array.from(prompt_content.children).includes(e.target)){
                closePopup()
                resolve(false)
            }
        }
    })
}

function alertPopup(title, text, delay=3000){
    prompt = document.getElementById("alertPrompt")
    spans = prompt.getElementsByTagName("span")
    spans[0].innerHTML = title
    spans[1].innerHTML = text

    prompt.classList.remove("hidden")

    setTimeout(function(){
        prompt.classList.add("hidden")
    }, delay)
}

function showError(error){
    if(error == "CanceledByUser"){
        return;
    }
    error_id = Object.keys(error)[0]
    alertPopup(error_id, error[error_id])
}

function minimizeWindow(){
    webWindow.minimize()
}

function maximizeWindow(){
    webWindow.maximize()
    .then(_ => {
        document.getElementById("maximizeIcon").src = "icons/unmaximize.png"
    })
}

function unmaximizeWindow(){
    webWindow.unmaximize()
    .then(_ => {
        document.getElementById("maximizeIcon").src = "icons/maximize.png"
    })
}

function closeWindow(){
    webWindow.close()
}

function requestUserAttention(){
    webWindow.requestUserAttention(2)
}

document.getElementById("minimizeButton").onclick = minimizeWindow
document.getElementById("maximizeButton").onclick = function(){
    webWindow.isMaximized()
    .then(maximized => {
        if(maximized){
            unmaximizeWindow()
        }else{
            maximizeWindow()
        }
    })
}
document.getElementById("closeButton").onclick = closeWindow