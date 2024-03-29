var videoLinkField = document.getElementById("videoLinkField")
var videoBlock = document.getElementById("videoBlock")
var videoTitle = document.getElementById("videoTitle")
var videoViews = document.getElementById("videoViews")
var videoLikes = document.getElementById("videoLikes")
var videoComments = document.getElementById("videoComments")
var videoPreview = document.getElementById("videoPreview")
var continueLink = document.getElementById("continueLink")
var continueButton = document.getElementById("continueButton")


videoLinkField.oninput = function(){
    clearVideoData()
    checkVideoId(videoLinkField.value)
    .then((id) => {
        if(id != null){
            videoBlock.classList.remove("hidden")
            setVideoData(id)
            .then(result => {
                if (result == true){
                    videoLinkField.classList.remove("invalid")
                    window.currentFmt = null
                    window.savePath = null
                    window.start_seconds = null
                    window.end_seconds = null
                    getCurrentVideo()
                    .then((videoData) => {
                        loadVideoData(videoData)
                    })
                    .catch(error => {
                        showError(error)
                    })
                }else{
                    videoBlock.classList.add("hidden")
                    continueButton.classList.add("inactive")
                    continueLink.href = "#"
                    videoLinkField.classList.add("invalid")
                }
            })
            .catch(error => {
                videoBlock.classList.add("hidden")
                continueButton.classList.add("inactive")
                continueLink.href = "#"
                videoLinkField.classList.add("invalid")
                showError(error)
            })
        }else{
            videoBlock.classList.add("hidden")
            continueButton.classList.add("inactive")
            continueLink.href = "#"
            videoLinkField.classList.add("invalid")
        }
    })
}

function findClosestPreview(previews){
    let bestDifference = Infinity
    let bestIndex = 0
    for (let i = 0; i < previews.length; i++) {
        const preview = previews[i];
        if (preview.url != null && preview.width != null && preview.height != null){
            const difference = (window.screen.width-preview.width)+(window.screen.height-preview.height)
            if ( difference > 0 && difference < bestDifference){
                bestDifference = difference
                bestIndex = i
            }
        }
    }
    return previews[bestIndex].url
}

function clearVideoData(){
    videoTitle.innerHTML = ""
    videoViews.innerHTML = ""
    videoLikes.innerHTML = ""
    videoComments.innerHTML = ""
    videoPreview.src = "data:image/gif;base64,R0lGODlhAQABAPcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACH5BAEAAP8ALAAAAAABAAEAAAgEAP8FBAA7"
}

function loadVideoData(videoData){
    continueButton.classList.remove("inactive")
    continueLink.href = "options.html"
    if (videoData.title != null){
        videoTitle.innerHTML = videoData.title
    }

    if (videoData.views_count != null){
        videoViews.innerHTML = videoData.views_count
    }else{
        videoViews.innerHTML = "Views unavailable"
    }

    if (videoData.likes_count != null){
        videoLikes.innerHTML = videoData.likes_count
    }else{
        videoLikes.innerHTML = "Likes unavailable"
    }

    if (videoData.comments_count != null){
        videoComments.innerHTML = videoData.comments_count
    }else{
        videoComments.innerHTML = "Comments unavailable"
    }

    if (videoData.previews != null && videoData.previews.length > 0){
        let bestPreviewLink = findClosestPreview(videoData.previews)
        if (bestPreviewLink != null){
            videoPreview.src = bestPreviewLink
        }
    }
}

getCurrentVideo()
.then(videoData => {
    console.log(videoData)
    videoLinkField.value = `https://www.youtube.com/watch?v=${videoData.id}`
    loadVideoData(videoData)
    videoBlock.classList.remove("hidden")
})
.catch(error => {
    console.log("Can't get current video, maybe it's not loaded yet")
})