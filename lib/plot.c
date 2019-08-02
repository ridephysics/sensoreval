#define PY_SSIZE_T_CLEAN
#include <Python.h>

#include <sensoreval.h>

static PyObject* import(const char *name) {
    PyObject *pyname;
    PyObject *module;

    pyname = PyUnicode_DecodeFSDefault(name);
    if (!pyname) {
        fprintf(stderr, "can't decode: %s\n", name);
        return NULL;
    }

    module = PyImport_Import(pyname);
    Py_DECREF(pyname);
    if (!module) {
        fprintf(stderr, "can't import: %s\n", name);
        PyErr_Print();
    }

    return module;
}

static PyObject* plt_subplots(PyObject *plt, size_t nrows) {
    int rc;
    PyObject *subplots = NULL;
    PyObject *args = NULL;
    PyObject *ret = NULL;

    subplots = PyObject_GetAttrString(plt, "subplots");
    if (!subplots || !PyCallable_Check(subplots)) {
        fprintf(stderr, "can't find subplots\n");
        goto err;
    }

    args = PyTuple_New(3);
    if (!args) {
        fprintf(stderr, "can't create argument tuple\n");
        goto err;
    }

    rc = PyTuple_SetItem(args, 0, PyLong_FromLong(nrows));
    if (rc < 0) {
        fprintf(stderr, "can't set arg 0: %d\n", rc);
        goto err;
    }

    rc = PyTuple_SetItem(args, 1, PyLong_FromLong(1));
    if (rc < 0) {
        fprintf(stderr, "can't set arg 1: %d\n", rc);
        goto err;
    }

    rc = PyTuple_SetItem(args, 2, Py_True);
    if (rc < 0) {
        fprintf(stderr, "can't set arg 2: %d\n", rc);
        goto err;
    }

    ret = PyObject_CallObject(subplots, args);
    Py_DECREF(args);
    args = NULL;

    if (!ret) {
        fprintf(stderr, "plt.subplots failed\n");
        PyErr_Print();
        goto err;
    }

    return ret;

err:
    if (ret)
        Py_DECREF(ret);
    if (args)
        Py_DECREF(args);
    if (subplots)
        Py_DECREF(subplots);

    return NULL;
}

static int plt_show(PyObject *plt) {
    int ret = -1;
    PyObject *show = NULL;
    PyObject *pyret = NULL;

    show = PyObject_GetAttrString(plt, "show");
    if (!show || !PyCallable_Check(show)) {
        fprintf(stderr, "can't find show\n");
        goto final;
    }

    pyret = PyObject_CallObject(show, NULL);
    if (!pyret) {
        fprintf(stderr, "plt.show failed\n");
        PyErr_Print();
        goto final;
    }

    ret = 0;

final:
    if (pyret)
        Py_DECREF(pyret);
    if (show)
        Py_DECREF(show);

    return ret;
}

static int axis_plot(PyObject *axis, PyObject *time, PyObject *data) {
    int ret = -1;
    int rc;
    PyObject *plot = NULL;
    PyObject *args = NULL;
    PyObject *pyret = NULL;

    plot = PyObject_GetAttrString(axis, "plot");
    if (!plot || !PyCallable_Check(plot)) {
        fprintf(stderr, "can't find plot\n");
        goto final;
    }

    args = PyTuple_New(2);
    if (!args) {
        fprintf(stderr, "can't create argument tuple\n");
        goto final;
    }

    rc = PyTuple_SetItem(args, 0, time);
    if (rc < 0) {
        fprintf(stderr, "can't set arg 0: %d\n", rc);
        goto final;
    }

    rc = PyTuple_SetItem(args, 1, data);
    if (rc < 0) {
        fprintf(stderr, "can't set arg 1: %d\n", rc);
        goto final;
    }

    pyret = PyObject_CallObject(plot, args);
    if (!pyret) {
        fprintf(stderr, "plt.plot failed\n");
        PyErr_Print();
        goto final;
    }

    ret = 0;

final:
    if (pyret)
        Py_DECREF(pyret);
    if (args)
        Py_DECREF(args);
    if (plot)
        Py_DECREF(plot);

    return ret;
}

static PyObject* np_array(PyObject *np, PyObject *list) {
    int rc;
    PyObject *array = NULL;
    PyObject *args = NULL;
    PyObject *pyret = NULL;

    array = PyObject_GetAttrString(np, "array");
    if (!array || !PyCallable_Check(array)) {
        fprintf(stderr, "can't find array\n");
        goto final;
    }

    args = PyTuple_New(1);
    if (!args) {
        fprintf(stderr, "can't create argument tuple\n");
        goto final;
    }

    rc = PyTuple_SetItem(args, 0, list);
    if (rc < 0) {
        fprintf(stderr, "can't set arg 0: %d\n", rc);
        goto final;
    }

    pyret = PyObject_CallObject(array, args);
    if (!pyret) {
        fprintf(stderr, "np.array failed\n");
        PyErr_Print();
        goto final;
    }

final:
    if (args)
        Py_DECREF(args);
    if (array)
        Py_DECREF(array);

    return pyret;
}

int sensoreval_plot(size_t nplots, sensoreval_plot_getdata_t getdata, void *ctx) {
    int rc;
    int ret = -1;
    size_t i;
    wchar_t *program = NULL;
    PyObject *plt = NULL;
    PyObject *np = NULL;
    PyObject *subplots_ret = NULL;
    PyObject *timearr = NULL;
    PyObject *axes = NULL;
    PyObject *tmp = NULL;

    program = Py_DecodeLocale("python", NULL);
    if (!program) {
        fprintf(stderr, "can't decode program name\n");
        return -1;
    }

    Py_SetProgramName(program);
    Py_Initialize();

    plt = import("matplotlib.pyplot");
    if (!plt) {
        fprintf(stderr, "can't import plt\n");
        goto finalize;
    }

    np = import("numpy");
    if (!np) {
        fprintf(stderr, "can't import numpy\n");
        goto finalize;
    }

    PyRun_SimpleString("import sys\nsys.argv.append('python')\n");

    subplots_ret = plt_subplots(plt, nplots);
    if (!subplots_ret) {
        fprintf(stderr, "plt.subplots failed\n");
        goto finalize;
    }

    timearr = getdata(0, ctx);
    if (!timearr) {
        fprintf(stderr, "can't get time array\n");
        goto finalize;
    }

    tmp = np_array(np, timearr);
    if (!tmp) {
        fprintf(stderr, "can't get time np.array\n");
        goto finalize;
    }
    Py_DECREF(timearr);
    timearr = tmp;
    tmp = NULL;

    axes = PyTuple_GetItem(subplots_ret, 1);
    if (!axes) {
        fprintf(stderr, "can't get axes\n");
        goto finalize;
    }

    for (i=0; i<nplots; i++) {
        PyObject *axis = NULL;
        PyObject *data = NULL;

        data = getdata(i + 1, ctx);
        if (!data) {
            fprintf(stderr, "can't get data for plot %zu\n", i + 1);
            goto finalize;
        }

        tmp = np_array(np, data);
        if (!tmp) {
            fprintf(stderr, "can't get time np.array\n");
            PyErr_Print();
            Py_DECREF(data);
            goto finalize;
        }
        Py_DECREF(data);
        data = tmp;
        tmp = NULL;

        if (nplots > 1) {
            axis = PySequence_GetItem(axes, i);
            if (!axis) {
                fprintf(stderr, "can't get axis from array for plot %zu\n", i + 1);
                PyErr_Print();
                Py_DECREF(data);
                goto finalize;
            }
        }
        else {
            axis = axes;
        }

        rc = axis_plot(axis, timearr, data);
        Py_DECREF(data);
        if (nplots > 1)
            Py_DECREF(axis);
        if (rc) {
            fprintf(stderr, "can't plot axis for %zu\n", i + 1);
            goto finalize;
        }
    }

    rc = plt_show(plt);
    if (rc) {
        fprintf(stderr, "plt_show failed: %d\n", rc);
        goto finalize;
    }

    ret = 0;

finalize:
    if (tmp)
        Py_DECREF(tmp);
    if (timearr)
        Py_DECREF(timearr);
    if (subplots_ret)
        Py_DECREF(subplots_ret);
    if (np)
        Py_DECREF(np);
    if (plt)
        Py_DECREF(plt);
    if (program)
        PyMem_RawFree(program);

    rc = Py_FinalizeEx();
    if (rc < 0) {
        fprintf(stderr, "Py_FinalizeEx: %d\n", rc);
    }

    return ret;
}
