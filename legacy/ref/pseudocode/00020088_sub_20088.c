/* ea:   0x00020088 */
/* name: sub_20088 */
/* size: 1780 */
int __fastcall sub_20088(int a1, int a2, _DWORD *a3)
{
  void **v6; // r4
  int i; // r5
  int v8; // r0
  int v9; // r0
  int v10; // r0
  int v11; // r0
  int v12; // r0
  int v13; // r0
  int v14; // r0
  int v15; // r0
  int v16; // r0
  int v17; // r0
  int v18; // r0
  int v19; // r0
  int v20; // r0
  int v21; // r0
  int v22; // r0
  int v23; // r0
  int v24; // r0
  int v25; // r0
  int v26; // r0
  int v27; // r0
  int v28; // r0
  int v29; // r0
  int v30; // r0
  int v31; // r0
  int v32; // r0
  int v33; // r0
  int v34; // r0
  int v35; // r0
  int v36; // r0
  int v37; // r0
  int v38; // r0
  int v39; // r0
  int v40; // r0
  int v41; // r0
  int v42; // r0
  int v43; // r0
  int v44; // r0
  int v45; // r0
  int v46; // r0
  int v47; // r0
  int v48; // r0
  int v49; // r0
  int v50; // r0
  int v51; // r7
  void **v52; // r6
  int v53; // r0
  int v54; // r0
  int v55; // r5
  void *v56; // r0
  void *v57; // r4
  bool v58; // zf
  _WORD *v60; // [sp+4h] [bp-274h] BYREF
  int (__fastcall **v61)(void *); // [sp+8h] [bp-270h] BYREF
  char v62; // [sp+Ch] [bp-26Ch]
  int v63; // [sp+10h] [bp-268h]
  _DWORD v64[12]; // [sp+20h] [bp-258h] BYREF
  void *v65[3]; // [sp+50h] [bp-228h] BYREF
  void *v66; // [sp+5Ch] [bp-21Ch] BYREF
  void *v67; // [sp+68h] [bp-210h] BYREF
  void *v68; // [sp+74h] [bp-204h] BYREF
  void *v69; // [sp+80h] [bp-1F8h] BYREF
  void *v70; // [sp+8Ch] [bp-1ECh] BYREF
  void *v71; // [sp+98h] [bp-1E0h] BYREF
  void *v72; // [sp+A4h] [bp-1D4h] BYREF
  void *v73; // [sp+B0h] [bp-1C8h] BYREF
  void *v74; // [sp+BCh] [bp-1BCh] BYREF
  void *v75; // [sp+C8h] [bp-1B0h] BYREF
  void *v76; // [sp+D4h] [bp-1A4h] BYREF
  void *v77; // [sp+E0h] [bp-198h] BYREF
  void *v78; // [sp+ECh] [bp-18Ch] BYREF
  void *v79; // [sp+F8h] [bp-180h] BYREF
  void *v80; // [sp+104h] [bp-174h] BYREF
  void *v81; // [sp+110h] [bp-168h] BYREF
  void *v82; // [sp+11Ch] [bp-15Ch] BYREF
  void *v83; // [sp+128h] [bp-150h] BYREF
  void *v84; // [sp+134h] [bp-144h] BYREF
  void *v85; // [sp+140h] [bp-138h] BYREF
  void *v86; // [sp+14Ch] [bp-12Ch] BYREF
  void *v87; // [sp+158h] [bp-120h] BYREF
  void *v88; // [sp+164h] [bp-114h] BYREF
  void *v89; // [sp+170h] [bp-108h] BYREF
  void *v90; // [sp+17Ch] [bp-FCh] BYREF
  void *v91; // [sp+188h] [bp-F0h] BYREF
  void *v92; // [sp+194h] [bp-E4h] BYREF
  void *v93; // [sp+1A0h] [bp-D8h] BYREF
  void *v94; // [sp+1ACh] [bp-CCh] BYREF
  void *v95; // [sp+1B8h] [bp-C0h] BYREF
  void *v96; // [sp+1C4h] [bp-B4h] BYREF
  void *v97; // [sp+1D0h] [bp-A8h] BYREF
  void *v98; // [sp+1DCh] [bp-9Ch] BYREF
  void *v99; // [sp+1E8h] [bp-90h] BYREF
  void *v100; // [sp+1F4h] [bp-84h] BYREF
  void *v101; // [sp+200h] [bp-78h] BYREF
  void *v102; // [sp+20Ch] [bp-6Ch] BYREF
  void *v103; // [sp+218h] [bp-60h] BYREF
  void *v104; // [sp+224h] [bp-54h] BYREF
  void *v105; // [sp+230h] [bp-48h] BYREF
  void *v106; // [sp+23Ch] [bp-3Ch] BYREF
  void *v107; // [sp+248h] [bp-30h] BYREF

  sub_1E61C((int)v64);
  sub_1DCEC((int)&v61);
  v6 = v65;
  v61 = &off_6C938;
  for ( i = 43; i != 0; --i )
  {
    sub_1333C(v6);
    v6 += 3;
  }
  if ( (unsigned __int8)sub_63BE8(v64, a2) != 0 )
  {
    v8 = sub_12F04(aBgPcx);
    sub_1318C(v65, (char *)aBgPcx, (int)&aBgPcx[v8]);
    v9 = sub_12F04(aNumberPcx);
    sub_1318C(&v66, (char *)aNumberPcx, (int)&aNumberPcx[v9]);
    v10 = sub_12F04(aFlagRoadPcx);
    sub_1318C(&v67, (char *)aFlagRoadPcx, (int)&aFlagRoadPcx[v10]);
    v11 = sub_12F04(aLandmarkCity1P);
    sub_1318C(&v68, (char *)aLandmarkCity1P, (int)&aLandmarkCity1P[v11]);
    v12 = sub_12F04(aLandmarkCity2P);
    sub_1318C(&v69, (char *)aLandmarkCity2P, (int)&aLandmarkCity2P[v12]);
    v13 = sub_12F04(aLandmarkCity3P);
    sub_1318C(&v70, (char *)aLandmarkCity3P, (int)&aLandmarkCity3P[v13]);
    v14 = sub_12F04(aLandmarkCity4P);
    sub_1318C(&v71, (char *)aLandmarkCity4P, (int)&aLandmarkCity4P[v14]);
    v15 = sub_12F04(aLandmarkPort1P);
    sub_1318C(&v72, (char *)aLandmarkPort1P, (int)&aLandmarkPort1P[v15]);
    v16 = sub_12F04(aLandmarkPort2P);
    sub_1318C(&v73, (char *)aLandmarkPort2P, (int)&aLandmarkPort2P[v16]);
    v17 = sub_12F04(aLandmarkPort3P);
    sub_1318C(&v74, (char *)aLandmarkPort3P, (int)&aLandmarkPort3P[v17]);
    v18 = sub_12F04(aLandmarkPort4P);
    sub_1318C(&v75, (char *)aLandmarkPort4P, (int)&aLandmarkPort4P[v18]);
    v19 = sub_12F04(aPlayerKatharin);
    sub_1318C(&v76, (char *)aPlayerKatharin, (int)&aPlayerKatharin[v19]);
    v20 = sub_12F04(aPlayerDrakePcx);
    sub_1318C(&v77, (char *)aPlayerDrakePcx, (int)&aPlayerDrakePcx[v20]);
    v21 = sub_12F04(aPlayerKiddPcx);
    sub_1318C(&v78, (char *)aPlayerKiddPcx, (int)&aPlayerKiddPcx[v21]);
    v22 = sub_12F04(aPlayerTichyPcx);
    sub_1318C(&v79, (char *)aPlayerTichyPcx, (int)&aPlayerTichyPcx[v22]);
    v23 = sub_12F04(aPlayerPandaPcx);
    sub_1318C(&v80, (char *)aPlayerPandaPcx, (int)&aPlayerPandaPcx[v23]);
    v24 = sub_12F04(aFlagKatharinaP);
    sub_1318C(&v81, (char *)aFlagKatharinaP, (int)&aFlagKatharinaP[v24]);
    v25 = sub_12F04(aFlagDrakePcx);
    sub_1318C(&v82, (char *)aFlagDrakePcx, (int)&aFlagDrakePcx[v25]);
    v26 = sub_12F04(aFlagKiddPcx);
    sub_1318C(&v83, (char *)aFlagKiddPcx, (int)&aFlagKiddPcx[v26]);
    v27 = sub_12F04(aFlagTichyPcx);
    sub_1318C(&v84, (char *)aFlagTichyPcx, (int)&aFlagTichyPcx[v27]);
    v28 = sub_12F04(aFlagPandaPcx);
    sub_1318C(&v85, (char *)aFlagPandaPcx, (int)&aFlagPandaPcx[v28]);
    v29 = sub_12F04(aShipAllPcx);
    sub_1318C(&v91, (char *)aShipAllPcx, (int)&aShipAllPcx[v29]);
    v30 = sub_12F04(aTroopAllPcx);
    sub_1318C(&v92, (char *)aTroopAllPcx, (int)&aTroopAllPcx[v30]);
    v31 = sub_12F04(aFaceKatharinaP);
    sub_1318C(&v86, (char *)aFaceKatharinaP, (int)&aFaceKatharinaP[v31]);
    v32 = sub_12F04(aFaceDrakePcx);
    sub_1318C(&v87, (char *)aFaceDrakePcx, (int)&aFaceDrakePcx[v32]);
    v33 = sub_12F04(aFaceKiddPcx);
    sub_1318C(&v88, (char *)aFaceKiddPcx, (int)&aFaceKiddPcx[v33]);
    v34 = sub_12F04(aFaceTichyPcx);
    sub_1318C(&v89, (char *)aFaceTichyPcx, (int)&aFaceTichyPcx[v34]);
    v35 = sub_12F04(aFacePandaPcx);
    sub_1318C(&v90, (char *)aFacePandaPcx, (int)&aFacePandaPcx[v35]);
    v36 = sub_12F04(aRoleShadowPcx);
    sub_1318C(&v93, (char *)aRoleShadowPcx, (int)&aRoleShadowPcx[v36]);
    v37 = sub_12F04(aRoleStatusPcx);
    sub_1318C(&v94, (char *)aRoleStatusPcx, (int)&aRoleStatusPcx[v37]);
    v38 = sub_12F04(aFaceSavagerPcx);
    sub_1318C(&v95, (char *)aFaceSavagerPcx, (int)&aFaceSavagerPcx[v38]);
    v39 = sub_12F04(aTextGameoverPc);
    sub_1318C(&v96, (char *)aTextGameoverPc, (int)&aTextGameoverPc[v39]);
    v40 = sub_12F04(aTextGamerwinPc);
    sub_1318C(&v97, (char *)aTextGamerwinPc, (int)&aTextGamerwinPc[v40]);
    v41 = sub_12F04(aFlagWildwoodPc);
    sub_1318C(&v98, (char *)aFlagWildwoodPc, (int)&aFlagWildwoodPc[v41]);
    v42 = sub_12F04(aFlagHolidaypla);
    sub_1318C(&v99, (char *)aFlagHolidaypla, (int)&aFlagHolidaypla[v42]);
    v43 = sub_12F04(aFlagBranchroad);
    sub_1318C(&v100, (char *)aFlagBranchroad, (int)&aFlagBranchroad[v43]);
    v44 = sub_12F04(aFlagBattlefiel);
    sub_1318C(&v101, (char *)aFlagBattlefiel, (int)&aFlagBattlefiel[v44]);
    v45 = sub_12F04(aStartKatharina);
    sub_1318C(&v102, (char *)aStartKatharina, (int)&aStartKatharina[v45]);
    v46 = sub_12F04(aStartDrakePcx);
    sub_1318C(&v103, (char *)aStartDrakePcx, (int)&aStartDrakePcx[v46]);
    v47 = sub_12F04(aStartKiddPcx);
    sub_1318C(&v104, (char *)aStartKiddPcx, (int)&aStartKiddPcx[v47]);
    v48 = sub_12F04(aStartTichyPcx);
    sub_1318C(&v105, (char *)aStartTichyPcx, (int)&aStartTichyPcx[v48]);
    v49 = sub_12F04(aStartPandaPcx);
    sub_1318C(&v106, (char *)aStartPandaPcx, (int)&aStartPandaPcx[v49]);
    v50 = sub_12F04(aDialoguePcx);
    sub_1318C(&v107, (char *)aDialoguePcx, (int)&aDialoguePcx[v50]);
    sub_1DDA4(a1 + 32, 0x2Bu);
    v51 = 0;
    v52 = v65;
    while ( 1 )
    {
      v60 = *v52;
      v53 = sub_1E5CC(&v60);
      v54 = sub_1E84C((int)v64, v53);
      v55 = v54;
      if ( v54 == 0 )
        __debugbreak();
      v56 = operator new(*(_DWORD *)(v54 + 4));
      v57 = v56;
      if ( v56 == nullptr )
        break;
      sub_63CE8((int)v64, v56);
      v58 = v51 == 104;
      if ( v51 != 104 )
        v58 = v51 == 108;
      if ( v58 )
      {
        v62 = 0;
      }
      else
      {
        v62 = 1;
        v63 = 16;
      }
      *(_DWORD *)(*a3 + v51) = sub_645B4(&v61, v57, *(_DWORD *)(v55 + 4));
      operator delete(v57);
      if ( *(_DWORD *)(*a3 + v51) == 0 )
        break;
      v51 += 4;
      v52 += 3;
      if ( v51 >= 172 )
        goto LABEL_15;
    }
    sub_11634((int)v65, 12, 43, sub_12EA0);
    sub_1EA14(&v61);
    sub_1E798(v64);
    return 0;
  }
  else
  {
LABEL_15:
    sub_11634((int)v65, 12, 43, sub_12EA0);
    sub_1EA14(&v61);
    sub_1E798(v64);
    return 1;
  }
}

