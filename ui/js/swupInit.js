const options = {
    linkSelector:
      'a:not([data-no-swup])'
  };
  
const swup = new Swup(options);

function initDependencies(){
    if(document.getElementById("generalJS") == null){
        let general = document.createElement("script")
        general.setAttribute("defer", "")
        general.setAttribute("id", "generalJS")
        general.setAttribute("src", "js/general.js")
        document.head.appendChild(general)
    }
    let js = document.createElement("script")
        js.setAttribute("defer", "")
        js.setAttribute("id", "pageJS")

    switch (window.location.href.split("/").at(-1)) {
        case "index.html":
            js.setAttribute("src", "js/index.js")
            break;
        case "options.html":
            js.setAttribute("src", "js/options.js")
            break;
        case "progress.html":
            js.setAttribute("src", "js/progress.js")
    }
    document.head.appendChild(js)
}

function unloadDependencies(){
    let js = document.getElementById("pageJS")
    let general = document.getElementById("generalJS")
    if (js != null){
        js.remove()
    }
    if (general != null){
        general.remove()
    }
}

swup.on('contentReplaced', initDependencies);
swup.on('willReplaceContent', unloadDependencies);