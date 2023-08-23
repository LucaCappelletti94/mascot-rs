"""Retrieve the data from Zenodo."""
from downloaders import BaseDownloader
import os

def retrieve_zenodo_data():
    """Retrieve the data from Zenodo."""
    downloader = BaseDownloader(
        process_number=1,
    )

    urls = [
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-LIBRARY.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-SELLECKCHEM-FDA-PART1.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-SELLECKCHEM-FDA-PART2.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-PRESTWICKPHYTOCHEM.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-NIH-CLINICALCOLLECTION1.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-NIH-CLINICALCOLLECTION2.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-NIH-NATURALPRODUCTSLIBRARY.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-NIH-NATURALPRODUCTSLIBRARY_ROUND2_POSITIVE.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-NIH-NATURALPRODUCTSLIBRARY_ROUND2_NEGATIVE.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-NIH-SMALLMOLECULEPHARMACOLOGICALLYACTIVE.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-FAULKNERLEGACY.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-EMBL-MCF.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-COLLECTIONS-PESTICIDES-POSITIVE.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-COLLECTIONS-PESTICIDES-NEGATIVE.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/MMV_POSITIVE.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/MMV_NEGATIVE.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/LDB_POSITIVE.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/LDB_NEGATIVE.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-NIST14-MATCHES.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-COLLECTIONS-MISC.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-MSMLS.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/PSU-MSMLS.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/BILELIB19.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/DEREPLICATOR_IDENTIFIED_LIBRARY.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/PNNL-LIPIDS-POSITIVE.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/PNNL-LIPIDS-NEGATIVE.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/MIADB.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/HCE-CELL-LYSATE-LIPIDS.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/UM-NPDC.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-NUTRI-METAB-FEM-POS.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-NUTRI-METAB-FEM-NEG.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-SCIEX-LIBRARY.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-IOBA-NHC.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/BERKELEY-LAB.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/IQAMDB.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-SAM-SIK-KANG-LEGACY-LIBRARY.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-D2-AMINO-LIPID-LIBRARY.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/DRUGS-OF-ABUSE-LIBRARY.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/ECG-ACYL-AMIDES-C4-C24-LIBRARY.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/ECG-ACYL-ESTERS-C4-C24-LIBRARY.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-IIMN-PROPOGATED.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/GNPS-SUSPECTLIST.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/BMDMS-NP.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/MASSBANK.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/MASSBANKEU.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/MONA.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/RESPECT.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/HMDB.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/CASMI.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/SUMNER.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/BIRMINGHAM-UHPLC-MS-POS.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/BIRMINGHAM-UHPLC-MS-NEG.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/ALL_GNPS.mgf",
        "https://gnps-external.ucsd.edu/gnpslibrary/ALL_GNPS_NO_PROPOGATED.mgf",
        "https://zenodo.org/record/8275685/files/20220513_PMA_DBGI_01_04_001.mgf",
        "https://zenodo.org/record/8275685/files/20220513_PMA_DBGI_01_04_001.mzML_chromatograms_deconvoluted_deisotoped_filtered_enpkg_sirius.mgf",
        "https://zenodo.org/record/8275685/files/20220513_PMA_DBGI_01_04_002.mgf",
        "https://zenodo.org/record/8275685/files/20220513_PMA_DBGI_01_04_002.mzML_chromatograms_deconvoluted_deisotoped_filtered_enpkg_sirius.mgf",
        "https://zenodo.org/record/8275685/files/20220513_PMA_DBGI_01_04_003.mgf",
        "https://zenodo.org/record/8275685/files/20220513_PMA_DBGI_01_04_003.mzML_chromatograms_deconvoluted_deisotoped_filtered_enpkg_sirius.mgf",
        "https://zenodo.org/record/8275685/files/20220513_PMA_DBGI_01_04_004.mgf",
        "https://zenodo.org/record/8275685/files/20220513_PMA_DBGI_01_04_004.mzML_chromatograms_deconvoluted_deisotoped_filtered_enpkg_sirius.mgf",
        "https://zenodo.org/record/8275685/files/20220513_PMA_DBGI_01_04_005.mgf",
        "https://zenodo.org/record/8275685/files/20220513_PMA_DBGI_01_04_005.mzML_chromatograms_deconvoluted_deisotoped_filtered_enpkg_sirius.mgf",
        "https://zenodo.org/record/8275685/files/20220513_PMA_DBGI_01_04_006.mgf",
        "https://zenodo.org/record/8275685/files/20220513_PMA_DBGI_01_04_006.mzML_chromatograms_deconvoluted_deisotoped_filtered_enpkg_sirius.mgf",
        "https://zenodo.org/record/8275685/files/20220513_PMA_DBGI_01_04_007.mgf",
        "https://zenodo.org/record/8275685/files/20220513_PMA_DBGI_01_04_007.mzML_chromatograms_deconvoluted_deisotoped_filtered_enpkg_sirius.mgf",
        "https://zenodo.org/record/8275685/files/20220513_PMA_DBGI_01_04_008.mgf",
        "https://zenodo.org/record/8275685/files/20220513_PMA_DBGI_01_04_008.mzML_chromatograms_deconvoluted_deisotoped_filtered_enpkg_sirius.mgf",
        "https://zenodo.org/record/8275685/files/20220513_PMA_DBGI_01_04_009.mgf",
        "https://zenodo.org/record/8275685/files/20220513_PMA_DBGI_01_04_009.mzML_chromatograms_deconvoluted_deisotoped_filtered_enpkg_sirius.mgf",
        "https://zenodo.org/record/8275685/files/20220513_PMA_DBGI_01_04_010.mgf",
        "https://zenodo.org/record/8275685/files/20220513_PMA_DBGI_01_04_010.mzML_chromatograms_deconvoluted_deisotoped_filtered_enpkg_sirius.mgf",
        "https://zenodo.org/record/8275685/files/mapp_batch_000052.mgf",
        "https://zenodo.org/record/8275685/files/mapp_batch_000052_sirius.mgf",
        "https://zenodo.org/record/8275685/files/mapp_batch_00059.mgf",
        "https://zenodo.org/record/8275685/files/mapp_batch_00059_sirius.mgf",
    ]

    downloader.download(
        urls,
        paths=[
            os.path.join("tests", "data", url.split("/")[-1])
            for url in urls
        ]
    )


if __name__ == "__main__":
    retrieve_zenodo_data()
