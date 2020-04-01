# get number of plots
nplots = load_data()
fig, plots = plt.subplots(nplots, sharex=True)
if nplots == 1:
    plots = [plots]

# load time data
x = np.array(load_data())
plotdata = []

def instr_titl():
    pid = load_data()
    title = load_data()

    plots[pid].set_title(title, x=-0.15, y=0.5)

def instr_plot():
    pid = load_data()
    color = load_data()
    data = load_data()

    plots[pid].plot(x, data, color=color)

instrs = {
    'titl': instr_titl,
    'plot': instr_plot,
}

# load figure data
while True:
    instr = load_data()
    if instr == None:
        break

    instrs[instr]()

fig.tight_layout()
plt.show()
