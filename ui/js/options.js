//I'm honored to present to you the worst js file I've ever written, it needs more refactoring than my life.
var saveDialog = window.__TAURI__.dialog.save
var videoTitle = document.getElementById("videoTitle")

var audioQualities = ["AUDIO_QUALITY_LOW", "AUDIO_QUALITY_MEDIUM", "AUDIO_QUALITY_HIGH"]
var qualityLabels = [
    "144p", "144p HDR", "144p60 HDR", "240p", 
    "240p HDR", "240p60 HDR", "360p", "360p HDR", 
    "360p60", "360p60 HDR", "480p", "480p HDR", 
    "480p60", "480p60 HDR", "720p", "720p50", 
    "720p60", "720p60 HDR", "1080p", "1080p50", 
    "1080p60", "1080p60 HDR", "1440p", "1440p60", 
    "1440p60 HDR", "2160p", "2160p60", "2160p60 HDR", 
    "4320p", "4320p60", "4320p60 HDR"]


//-----------------------UTILS--------------------------

function secondsToHumanTime(seconds){
    hours = Math.floor(seconds / 3600)
    remain = seconds % 3600
    minutes = Math.floor(remain / 60)
    seconds = remain % 60
    time = `${Math.round(hours)}:${Math.round(minutes)}:${Math.round(seconds)}`
    list = time.split(':')
    for (let i in list) {
        if (list[i].length == 1){
            list[i] = "0"+list[i]
        }
    }
    time = list[0]+':'+list[1]+':'+list[2]
    return time
}

function contentLengthToString(content_length){
    let units = ["Bytes", "Kb", "Mb", "Gb", "Tb", "Pb"]
    let current_unit = 0
    while (content_length/1024 > 1 && current_unit < units.length-1) {
        content_length = content_length/1024
        current_unit+=1
    }
    return `${content_length.toFixed(2)} ${units[current_unit]}`
}

function newSavePath(path){
    window.savePath = path
    document.getElementById("pathButton").innerHTML = path
}

function checkFFMpeg(){
    return new Promise(function(resolve, reject){
        if((window.start_seconds > 0 || window.end_seconds < window.videoDuration) || window.need_audio || window.savePath.endsWith(".mp3")){
            isFFmpegInstalled()
            .then(installed => {
                if(!installed){
                    resolve(false) 
                }else{
                    resolve(true)
                }
            })
        }else{
            resolve(true)
        }
    })
}

function allowDownloading(){
    checkFFMpeg()
    .then(result => {
        if(result){
            document.getElementById("downloadLink").href = "progress.html"
            document.getElementById("downloadButton").classList.remove("inactive")
        }else{
            document.getElementById("downloadLink").href = "#"
            document.getElementById("downloadButton").classList.add("inactive")
        }
    })
}

//-----------------------TimeLineSlider-----------------------------------

function createTimelineSlider(videoDuration){
    window.videoDuration = videoDuration
    points = document.getElementsByClassName("slider_draggable")
    var min = document.body.clientWidth/10
    var max = document.body.clientWidth/100*89
    var seconds_evaluating_coeficient = videoDuration/(max-min)
    console.log(seconds_evaluating_coeficient)
    var start_element = points[0]
    var draggable_start_position = 0
    var end_element = points[1]
    var currentlyDragging = null
    if(window.start_seconds == undefined || window.start_seconds == null || window.end_seconds == undefined || window.end_seconds == null){
        window.start_seconds = 0
        window.end_seconds = videoDuration
    }
    var slider_text_area = document.getElementById("sliderTime")
    var text_area_regex = /[0-9]{2}:[0-9]{2}:[0-9]{2}\s?-\s?[0-9]{2}:[0-9]{2}:[0-9]{2}/
    var text_area_reset_timeout;
    start_element.style.left = `${min}px`
    end_element.style.left = `${max}px`
    start_element.onmousedown = dragMouseDown
    end_element.onmousedown = dragMouseDown

    function dragMouseDown(e) {
        currentlyDragging =  e.target.parentNode
        e = e || window.event;
        e.preventDefault();
        draggable_start_position = e.clientX
        document.onmouseup = closeDragElement;
        document.onmousemove = dragElement;
    }

    function dragElement(e) {
        e = e || window.event;
        e.preventDefault();
        let move_to = currentlyDragging.offsetLeft - (draggable_start_position - e.clientX);
        draggable_start_position = e.clientX

        let current_min;    
        let current_max;
        let one_second_offset = 1/seconds_evaluating_coeficient;
        if(videoDuration > max-min){
            one_second_offset = start_element.clientWidth*0.4
        }

        if (currentlyDragging == start_element){
            current_min = min
            current_max = end_element.offsetLeft-one_second_offset
        }else{
            current_min = start_element.offsetLeft+one_second_offset
            current_max = max
        }

        if (e.clientX < current_min || move_to < current_min ){
            currentlyDragging.style.left = current_min + "px";
            setSeconds((current_min-document.body.clientWidth/10)*seconds_evaluating_coeficient) 
            return
        }
        if (e.clientX > current_max || move_to > current_max){
            currentlyDragging.style.left = current_max + "px";
            setSeconds((current_max-document.body.clientWidth/10)*seconds_evaluating_coeficient)
            return
        }
        currentlyDragging.style.left = move_to + "px";
        setSeconds((move_to-document.body.clientWidth/10)*seconds_evaluating_coeficient)
    }

    function closeDragElement() {
        draggable_start_position = 0
        document.onmouseup = null;
        document.onmousemove = null;
    }

    function setSeconds(seconds){
        if (currentlyDragging == start_element){
            window.start_seconds = seconds
        }else{
            window.end_seconds = seconds
        }
        allowDownloading()
        slider_text_area.value = `${secondsToHumanTime(Math.round(window.start_seconds))} - ${secondsToHumanTime(Math.round(window.end_seconds))}`
    }

    function resizeSlider(){
        min = document.body.clientWidth/10
        max = document.body.clientWidth/100*89
        seconds_evaluating_coeficient = videoDuration/(max-min)
        draggable_width = document.body.clientWidth/100
        start_element.style.left = `${window.start_seconds/seconds_evaluating_coeficient+document.body.clientWidth/10}px`
        end_element.style.left = `${window.end_seconds/seconds_evaluating_coeficient+document.body.clientWidth/10}px`
    }

    function initTextArea(){
        slider_text_area.oninput = function(){
            match = text_area_regex.exec(slider_text_area.value)
            if (match == null || match.length < 1 || parseTextArea() == null){
                if(text_area_reset_timeout != null){
                    clearTimeout(text_area_reset_timeout)
                    text_area_reset_timeout = null
                }
                text_area_reset_timeout = setTimeout(resetTextArea, 1000)
            }else{
                if (text_area_reset_timeout != null){
                    clearTimeout(text_area_reset_timeout)
                    text_area_reset_timeout = null
                }
                new_time = parseTextArea()
                window.start_seconds = new_time[0]
                window.end_seconds = new_time[1]
                resizeSlider()
                slider_text_area.value = `${secondsToHumanTime(Math.round(window.start_seconds))} - ${secondsToHumanTime(Math.round(window.end_seconds))}`
            }
        }
    }

    function resetTextArea(){
        slider_text_area.value = `${secondsToHumanTime(Math.round(window.start_seconds))} - ${secondsToHumanTime(Math.round(window.end_seconds))}`
        text_area_reset_timeout = null
    }

    function parseTextArea(){
        texts = slider_text_area.value.replaceAll(" ", "").split("-")
        start_text = texts[0].split(":")
        end_text = texts[1].split(":")
        start = Number(start_text[0])*3600+Number(start_text[1])*60+Number(start_text[2])
        end = Number(end_text[0])*3600+Number(end_text[1])*60+Number(end_text[2])
        if(start > end || start < 0 || end > videoDuration){
            return null
        }
        if(end-start == 0){
            if(videoDuration-end > 0){
                end+=1
            }else if(start > 0){
                start -=1
            }else{
                return null
            }
        }
        return [start, end]
    }

    window.onresize = resizeSlider
    initTextArea()
    resetTextArea()
    resizeSlider()
}


//----------------------VideoFormatsInDropdown------------------------

function setFmt(e){
    document.getElementById("dropdownButton").innerHTML = e.innerHTML
    window.currentFmt = Number(e.dataset.fmtId)
    console.log(window.loadedFmts[window.currentFmt]);
    for (let fmtElement of document.getElementById("dropdownItems").children) {
        fmtElement.classList.remove("active");
    }
    e.classList.add("active")
    
    if(!e.dataset.need_audio){
        window.need_audio = false
    }else{
        window.need_audio = true
    }
    let ext;
    if(window.currentFmt == 0){
        ext = "mp3"
    }else{
        ext = window.loadedFmts[window.currentFmt].extension
    }
    let videoName = document.getElementById("videoTitle").innerHTML
    if(window.savePath == null || window.savePath == undefined){
        getDefaultPath()
        .then((path) => {
            if(path == null){
                path = "C:"
            }
            newSavePath(`${path}\\${videoName}.${ext}`)
            allowDownloading()
        })
    }else{
        no_extension = window.savePath.split(".").slice(0, window.savePath.split(".").length-1)
        no_extension.push(ext)
        newSavePath(no_extension.join('.'))
        allowDownloading()
    }
}


function setVideoFormats(formats){
    formats = formats.filter(format => format.content_length)
    let labels_in = new Object
    let dropdownElement = document.getElementById("dropdownItems")
    let formatsElements = []
    bestAudio = formats
        .filter(format => format.includes_video == false)
        .sort((a,b) => audioQualities.indexOf(b.audio_quality)-audioQualities.indexOf(a.audio_quality))
        [0]
    bestVideo = formats
        .filter(format => format.includes_audio == false)
        .sort((a,b) => qualityLabels.indexOf(b.quality_label)-qualityLabels.indexOf(a.quality_label))
        [0]
    rest = formats
        .filter(format => format.includes_video == true)
        .sort((a,b) => qualityLabels.indexOf(a.quality_label)-qualityLabels.indexOf(b.quality_label))
    for (let i = 0; i < rest.length; i++) {
        const format = rest[i];
        const quality_label = format.quality_label
        if(labels_in[quality_label] == null){
            labels_in[quality_label] = i
        }else{
            if(format.bitrate != null && format.bitrate > rest[labels_in[quality_label].bitrate]){
                labels_in[quality_label] = i
            }}}
    mixedFormats = []
    for (const [_, value] of Object.entries(labels_in)) {
        mixedFormats.push(rest[value])
    }
    for (let i = 0; i < mixedFormats.length+2; i++) {
        let formatElement = document.createElement("span")
        formatElement.dataset.fmtId=i
        if( i > 0 && i < mixedFormats.length+1){
            let mixedFmt = mixedFormats[i-1]
            let fmtContentLength = mixedFmt.content_length
            if (!mixedFmt.includes_audio){
                fmtContentLength+=bestAudio.content_length
                formatElement.dataset.need_audio = true
            }
            formatElement.innerHTML = `${mixedFmt.quality_label}, ${contentLengthToString(fmtContentLength)}`
        }
        formatElement.className = "dropdown_item"
        formatsElements.push(formatElement)
    }
    formatsElements[0].innerHTML = `mp3, ${contentLengthToString(bestAudio.content_length)}`
    soundOnlyIcon = document.createElement("img")
    soundOnlyIcon.src = "icons/sound_only.png"
    soundOnlyIcon.className = "app_icon_small fmt_icon"
    formatsElements[0].appendChild(soundOnlyIcon)
    formatsElements[formatsElements.length-1].innerHTML = `${bestVideo.quality_label}, ${contentLengthToString(bestVideo.content_length)}`
    muteIcon = document.createElement("img")
    muteIcon.src = "icons/no_sound.png"
    muteIcon.className = "app_icon_small fmt_icon"
    formatsElements[formatsElements.length-1].appendChild(muteIcon)
    formatsElements.forEach(element => {
        element.onclick = function(e){setFmt(e.target)}
        dropdownElement.appendChild(element)
    });
    window.loadedFmts = [bestAudio].concat(mixedFormats)
    window.loadedFmts.push(bestVideo)
    if(window.currentFmt != undefined & window.currentFmt != null){
        console.log("Loading chosen format")
        setFmt(formatsElements[currentFmt])
    }else{
        console.log("Loading best-video+merged audio")
        setFmt(formatsElements[formatsElements.length-2])
    }
}

//------------------------LoadingGatheredDataToHTML---------------------------

function loadVideoData(){
    getCurrentVideo()
    .then((videoData) => {
        videoTitle.innerHTML = videoData.title
        createTimelineSlider(videoData.duration)
        document.getElementById("sliderTime").classList.remove("skeleton")
        setVideoFormats(videoData.streams)  
    })
    .catch(error => {
        showError(error)
    })
}

document.getElementById("pathButton").onclick = function(){
    let ext;
    let extName;
    if(window.currentFmt == 0){
        ext = "mp3"
        extName = "Audio"
    }else{
        ext = window.loadedFmts[window.currentFmt].extension
        extName = "Video"
    }
    saveDialog({
        defaultPath: window.savePath,
        filters: [{
            name: extName,
            extensions: [ext]
          }]
    }).then((response) => {
        if( response != null ){
            newSavePath(response)
        }
    })
}

document.getElementById("downloadButton").onclick = function(){
    checkFFMpeg()
    .then(result => {
        if(!result){
            confirmPopup(
                "FFMpeg not found!",
                "Media conversion, stream merging and video cutting are not available. Please install FFMpeg to enable these features.",
                "Ok"
            )
        }
    }) 
}

loadVideoData()