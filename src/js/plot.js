function axis2id(name) {
    if (name == "y") {
        return 0;
    }

    return parseInt(name.substring(1), 10) - 1;
}

function id2axis(id) {
    if (id == 0) {
        return "y";
    }

    return "y" + (id + 1);
}

function id2layoutaxis(id) {
    if (id == 0) {
        return "yaxis";
    }

    return "yaxis" + (id + 1);
}


var groups = [];
var rowconfig = {};
try {
    rowconfig = JSON.parse(window.localStorage.rowconfig);
}
catch(err) {
}

for (var traceid=0; traceid<data.length; traceid++) {
    var trace = data[traceid];
    const id = axis2id(trace.yaxis);
    if (!(id in groups)) {
        groups[id] = {
            "name": null,
            "traces": [],
        };

        const axisname = id2layoutaxis(id);
        if (axisname in layout && "title" in layout[axisname] && "text" in layout[axisname].title) {
            groups[id].name = layout[axisname].title.text;
        }
    }

    groups[id].traces.push(trace);
}

var plotsel = document.createElement("div");
plotsel.style.position = "fixed";
plotsel.style.width = "100%";
plotsel.style.height = "100%";
plotsel.style.top = 0;
plotsel.style.left = 0;
plotsel.style.background = "white";
plotsel.style.display = "none";
document.body.appendChild(plotsel);

var form = document.createElement("form");
plotsel.appendChild(form);

var list = document.createElement("div");
list.style.width = "100%";
list.style.height = "90%";
list.style.overflow = "auto";
form.appendChild(list);

for (var groupid=0; groupid<groups.length; groupid++) {
    var group = groups[groupid];

    var input = document.createElement("input");
    input.setAttribute("type", "checkbox");
    input.setAttribute("id", group.name);
    input.setAttribute("name", group.name);
    input.checked = true;
    list.appendChild(input);

    var label = document.createElement("label");
    label.setAttribute("for", group.name);
    label.innerText = group.name;
    list.appendChild(label);

    list.appendChild(document.createElement("br"));
}

var submit = document.createElement("input");
submit.setAttribute("type", "submit");
submit.setAttribute("value", "save");
submit.style.width = "100%";
submit.style.height = "10%";
submit.style.border = "none";
submit.style.background = "#00000011";
submit.style.cursor = "pointer";
form.appendChild(submit);

form.onsubmit = function(e) {
    for (var groupid=0; groupid<groups.length; groupid++) {
        var group = groups[groupid];
        var checked = list.querySelector("#"+group.name).checked;
        rowconfig[group.name] = checked;
    }

    window.localStorage.rowconfig = JSON.stringify(rowconfig);
    update_plot(true);

    plotsel.style.display = "none";
    document.body.style.overflowY = "auto";
    return false;
};

config["modeBarButtonsToAdd"] = [
    {
        name: 'plotsel',
        click: function(gd) {
            for (var groupid=0; groupid<groups.length; groupid++) {
                var group = groups[groupid];
                var checked = true;
                if (group.name in rowconfig && rowconfig[group.name]===false) {
                    checked = false;
                }
                list.querySelector("#"+group.name).checked = checked;
            }

            plotsel.style.display = "block";
            document.body.style.overflowY = "hidden";
        }
    },
];

function update_plot(redraw) {
    data.splice(0, data.length);
    var rowid = 0;
    for (var groupid=0; groupid<groups.length; groupid++) {
        var group = groups[groupid];

        if (group.name !== null && (group.name in rowconfig) && rowconfig[group.name]===false) {
            continue;
        }

        for (var traceid=0; traceid<group.traces.length; traceid++) {
            var trace = group.traces[traceid];
            trace.yaxis = id2axis(rowid);

            data.push(trace);
        }

        layout[id2layoutaxis(rowid)].title = group.name;

        rowid++;
    }

    layout.grid.rows = rowid;
    document.getElementById("plotly-div").style.height = Math.max(100.0 / 3.0 * rowid, 100.0) + "vh";

    if (redraw === true) {
        makeplot();
    }
}

update_plot(false);