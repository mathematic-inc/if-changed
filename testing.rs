macro_rules! git_test {
    ($($name:literal: [$($path:literal => $content:expr),*])* $(staged: [$($spath:literal => $scontent:expr),*])? $(working: [$($wdpath:literal => $wdcontent:expr),*])?) => {{
        let tempdir = ::tempfile::tempdir().unwrap();
        let repo = ::git2::Repository::init(tempdir.path()).unwrap();
        #[allow(unused_variables)]
        let signature = ::git2::Signature::new("Example User", "test@example.com", &::git2::Time::new(0, 0)).unwrap();
        #[allow(unused_variables, unused_mut)]
        let mut index = repo.index().unwrap();
        $({
            $({
                let path = tempdir.path().join($path);
                ::std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                ::std::fs::write(path, $content).unwrap();
            })*

            index
                .add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)
                .unwrap();
            index.write().unwrap();

            let oid = index.write_tree().unwrap();
            let tree = repo.find_tree(oid).unwrap();
            let parents = if let Ok(Ok(parent_commit)) = repo.head().map(|head| head.peel_to_commit()) {
                vec![parent_commit]
            } else {
                vec![]
            };
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                $name,
                &tree,
                &parents.iter().collect::<Vec<_>>(),
            ).unwrap();
        })*
        $($({
            let path = tempdir.path().join($spath);
            ::std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            ::std::fs::write(path, $scontent).unwrap();

            index
                .add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)
                .unwrap();
            index.write().unwrap();
        })*)?
        $($({
            let path = tempdir.path().join($wdpath);
            ::std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            ::std::fs::write(path, $wdcontent).unwrap();
        })*)?
        (tempdir, repo)
}}
}

pub(crate) use git_test;
