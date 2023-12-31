"""Script to evaluate which distributions may be used to model the data."""
import os
from glob import glob
from multiprocessing import Pool, cpu_count
from time import time

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import scipy
from tqdm.auto import tqdm


def task(args):
    """Test a distribution on a data set."""
    distribution, datapoints, data_name, n_mc_samples = args
    start = time()
    try:
        goodness = scipy.stats.goodness_of_fit(
            distribution,
            datapoints,
            n_mc_samples=n_mc_samples
        )
        pvalue = goodness.pvalue
        statistic = goodness.statistic
        parameters = goodness.fit_result.params._asdict()
        fig, axes = plt.subplots(1, 1, figsize=(10, 10), dpi=300)
        goodness.fit_result.plot()
        fig.savefig(f"distributions/{distribution.name}_{data_name}_{n_mc_samples}.png")
        plt.close(fig)
    except Exception as _exception:
        pvalue = 1.0
        statistic = np.inf
        parameters = dict()
    
    return {
        "name": distribution.name,
        "data_name": data_name,
        "pvalue": pvalue,
        "statistic":statistic,
        **parameters,
        "required_time": time() - start,
    }

if __name__ == "__main__":
    masses = []
    intensities = []
    n_mc_samples=1000

    for path in glob("tests/data/*.mgf"):
        with open(path, "r", encoding="utf8") as f:
            for line in f:
                if line.count(".") == 2 and line.count(" ") == 1:
                    mass, intensity = line.split(" ")
                    masses.append(float(mass))
                    intensities.append(float(intensity))
                    
    masses = np.array(masses)
    intensities = np.array(intensities)

    distributions = (
        scipy.stats.uniform,
        scipy.stats.norm,
        scipy.stats.loggamma,
        scipy.stats.gausshyper,
        scipy.stats.alpha,
        scipy.stats.anglit,
        scipy.stats.arcsine,
        scipy.stats.beta,
        scipy.stats.betaprime,
        scipy.stats.bradford,
        scipy.stats.burr,
        scipy.stats.burr12,
        scipy.stats.cauchy,
        scipy.stats.skewcauchy,
        scipy.stats.chi,
        scipy.stats.chi2,
        scipy.stats.dgamma,
        scipy.stats.dweibull,
        scipy.stats.expon,
        scipy.stats.exponweib,
        scipy.stats.exponpow,
        scipy.stats.fatiguelife,
        scipy.stats.fisk,
        scipy.stats.foldcauchy,
        scipy.stats.foldnorm,
        scipy.stats.f,
        scipy.stats.gamma,
        scipy.stats.genlogistic,
        scipy.stats.genpareto,
        scipy.stats.genextreme,
        scipy.stats.gengamma,
        scipy.stats.genhalflogistic,
        scipy.stats.geninvgauss,
        scipy.stats.gennorm,
        scipy.stats.gibrat,
        scipy.stats.gompertz,
        scipy.stats.halfcauchy,
        scipy.stats.halfnorm,
        scipy.stats.halflogistic,
        scipy.stats.invgamma,
        scipy.stats.invgauss,
        scipy.stats.invweibull,
        scipy.stats.johnsonsb,
        scipy.stats.johnsonsu,
        scipy.stats.ksone,
        scipy.stats.kstwobign,
        scipy.stats.laplace,
        scipy.stats.laplace_asymmetric,
        scipy.stats.levy_l,
        scipy.stats.levy,
        scipy.stats.logistic,
        scipy.stats.loglaplace,
        scipy.stats.loggamma,
        scipy.stats.lognorm,
        scipy.stats.loguniform,
        scipy.stats.maxwell,
        scipy.stats.mielke,
        scipy.stats.ncx2,
        scipy.stats.ncf,
        scipy.stats.nct,
        scipy.stats.norminvgauss,
        scipy.stats.pareto,
        scipy.stats.lomax,
        scipy.stats.powerlognorm,
        scipy.stats.powerlaw,
        scipy.stats.rdist,
        scipy.stats.rayleigh,
        scipy.stats.rice,
        scipy.stats.recipinvgauss,
        scipy.stats.semicircular,
        scipy.stats.studentized_range,
        scipy.stats.t,
        scipy.stats.trapezoid,
        scipy.stats.triang,
        scipy.stats.truncexpon,
        scipy.stats.truncnorm,
        scipy.stats.truncpareto,
        scipy.stats.truncweibull_min,
        scipy.stats.tukeylambda,
        scipy.stats.vonmises,
        scipy.stats.wald,
        scipy.stats.weibull_max,
        scipy.stats.weibull_min,
        scipy.stats.wrapcauchy,
    )
    os.makedirs("distributions", exist_ok=True)
    with Pool(cpu_count()) as p:
        results = list(tqdm(
            p.imap(
                task,
                (
                    (distribution, data, data_name, n_mc_samples)
                    for distribution in distributions
                    for data, data_name in ((masses, "Masses"), (intensities, "Intensities"))
                )
            ),
            total=len(distributions) * 2,
            desc="Distribution",
            leave=False
        ))
            
    results = pd.DataFrame(results)
    results.to_csv(f"data_distribution_tests_{n_mc_samples}.csv", index=False)