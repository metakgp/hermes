# -*- coding: utf-8 -*-
"""
Scraper for IITKGP's public student notice board
http://www.iitkgp.ac.in/for-students
"""
from __future__ import print_function
from os import path, environ as env
import json
import hashlib
from bs4 import BeautifulSoup
import requests
try:
    from urllib.parse import urljoin
except ImportError:
    from urlparse import urljoin

if __package__ is None:
    import sys
    sys.path.append(path.dirname(path.dirname(path.abspath(__file__))))
    from settings import load_env # pylint: disable-msg=E0611
else:
    from .settings import load_env

load_env()

REQUESTS_SESSION = requests.Session()
DIFF_NOTICES = 10

BASE_URL = 'http://www.iitkgp.ac.in/for-students'

def scrape_notice(notice_url):
    """
    Scrape method for each notice
    """
    notice_json = {}
    requests_response = REQUESTS_SESSION.get(notice_url, allow_redirects=False)
    try:
        if requests_response.headers['Content-Type'] == 'application/pdf':
            hash_md5 = hashlib.md5()
            notice_json['attachment'] = notice_url
            attachment_response = REQUESTS_SESSION.get(notice_json['attachment'], stream=True)
            attachment_response.raw.decode_content = True
            for chunk in iter(lambda: attachment_response.raw.read(4096), b""):
                hash_md5.update(chunk)
            notice_json['attachment_md5'] = hash_md5.hexdigest()
            notice_json['html'] = '<div></div>'
            notice_json['text'] = ''
        elif 'text/html' in requests_response.headers['Content-Type']:
            notice_json['html'] = '<div>' + notice_url + '</div>'
            notice_json['text'] = notice_url
    except KeyError:
        notice_json['html'] = '<div></div>'
        notice_json['text'] = ''
    return notice_json

def handle_notices_diff(notices):
    """
    Method to check for new/updated notices
    """
    new_notices = REQUESTS_SESSION.post(urljoin(env['VERITAS_URL'], "diff/public"), json=notices)
    new_notices = json.loads(new_notices.json())
    return new_notices

def scrape_public():
    """
    Scrape method for public noticeboard
    """
    print("Beginning to scrape public noticeboard")
    all_notices = []
    diffed_notices = 0
    requests_response = REQUESTS_SESSION.get(BASE_URL)
    soup = BeautifulSoup(requests_response.text, "html.parser")
    noticeboard = soup.find('div', {'class': 'right_box'})
    notices = noticeboard.find('ul').find_all('li')
    for notice in notices:
        notice_title = notice.get_text()
        notice_url = urljoin(BASE_URL, notice.find('a').get('href'))
        notice_json = scrape_notice(notice_url)
        notice_json['title'] = "%s notice: %s" % ("General", notice_title)
        all_notices.append(notice_json)
        diffed_notices += 1
        if env['FIRST_RUN'] == 'false' and diffed_notices == DIFF_NOTICES:
            break
    new_notices = handle_notices_diff(all_notices)
    print("Found %d new notices in public noticeboard (%d checked)" % (
        len(new_notices), diffed_notices))
    return new_notices


if __name__ == "__main__":
    scrape_public()
