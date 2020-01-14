# load time data
x = np.array(pickle.load(sys.stdin.buffer))
plotdata = []

# load figure data
while True:
    isdata = pickle.load(sys.stdin.buffer)
    if not isdata:
        break

    plotdata.append(pickle.load(sys.stdin.buffer))

# create plot
fig, plots = plt.subplots(len(plotdata), sharex=True)
if len(plotdata) == 1:
    plots = [plots]

for i in range(len(plotdata)):
    plots[i].plot(x, plotdata[i])

fig.tight_layout()

# show
plt.show()
