/* ea:   0x00033548 */
/* name: sub_33548 */
/* size: 1768 */
int __fastcall sub_33548(int a1, int a2, int a3)
{
  int v6; // r8
  void **v7; // r4
  int i; // r5
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
  int v40; // r5
  void *v41; // r0
  void *v42; // r4
  _BYTE *v43; // r4
  _BYTE *v44; // r4
  int v45; // r7
  void **v46; // r6
  int v47; // r0
  int v48; // r0
  int v49; // r4
  void *v50; // r0
  void *v51; // r5
  int v52; // r0
  int v53; // r0
  int v54; // r0
  int v55; // r5
  void *v56; // r0
  void *v57; // r4
  _BYTE *v59; // r4
  int (__fastcall **v60)(void *); // [sp+0h] [bp-1E8h] BYREF
  char v61; // [sp+4h] [bp-1E4h]
  int v62; // [sp+8h] [bp-1E0h]
  _WORD *v63; // [sp+18h] [bp-1D0h] BYREF
  _WORD *v64; // [sp+1Ch] [bp-1CCh] BYREF
  int v65; // [sp+20h] [bp-1C8h] BYREF
  _WORD *v66; // [sp+24h] [bp-1C4h] BYREF
  int v67; // [sp+28h] [bp-1C0h] BYREF
  _DWORD v68[12]; // [sp+30h] [bp-1B8h] BYREF
  _BYTE v69[8]; // [sp+60h] [bp-188h] BYREF
  void *v70[3]; // [sp+68h] [bp-180h] BYREF
  void *v71[3]; // [sp+74h] [bp-174h] BYREF
  void *v72; // [sp+80h] [bp-168h] BYREF
  void *v73; // [sp+8Ch] [bp-15Ch] BYREF
  void *v74; // [sp+98h] [bp-150h] BYREF
  void *v75; // [sp+A4h] [bp-144h] BYREF
  void *v76; // [sp+B0h] [bp-138h] BYREF
  void *v77; // [sp+BCh] [bp-12Ch] BYREF
  void *v78; // [sp+C8h] [bp-120h] BYREF
  void *v79; // [sp+D4h] [bp-114h] BYREF
  void *v80; // [sp+E0h] [bp-108h] BYREF
  void *v81; // [sp+ECh] [bp-FCh] BYREF
  void *v82; // [sp+F8h] [bp-F0h] BYREF
  void *v83; // [sp+104h] [bp-E4h] BYREF
  void *v84; // [sp+110h] [bp-D8h] BYREF
  void *v85; // [sp+11Ch] [bp-CCh] BYREF
  void *v86; // [sp+128h] [bp-C0h] BYREF
  void *v87; // [sp+134h] [bp-B4h] BYREF
  void *v88; // [sp+140h] [bp-A8h] BYREF
  void *v89; // [sp+14Ch] [bp-9Ch] BYREF
  void *v90; // [sp+158h] [bp-90h] BYREF
  void *v91; // [sp+164h] [bp-84h] BYREF
  void *v92; // [sp+170h] [bp-78h] BYREF
  void *v93; // [sp+17Ch] [bp-6Ch] BYREF
  void *v94; // [sp+188h] [bp-60h] BYREF
  void *v95; // [sp+194h] [bp-54h] BYREF
  void *v96; // [sp+1A0h] [bp-48h] BYREF
  void *v97; // [sp+1ACh] [bp-3Ch] BYREF
  _WORD *v98; // [sp+1B8h] [bp-30h] BYREF
  _BYTE v99[36]; // [sp+1C4h] [bp-24h] BYREF

  sub_1E61C((int)v68);
  sub_1DCEC((int)&v60);
  v6 = 29;
  v60 = &off_6C938;
  v7 = v70;
  for ( i = 29; i != 0; --i )
  {
    sub_1333C(v7);
    v7 += 3;
  }
  if ( (unsigned __int8)sub_63BE8(v68, a2) != 0 )
  {
    v9 = sub_12F04(aButtonFocusPcx);
    sub_1318C(v70, (char *)aButtonFocusPcx, (int)&aButtonFocusPcx[v9]);
    v10 = sub_12F04(aButtonMovePcx);
    sub_1318C(v71, (char *)aButtonMovePcx, (int)&aButtonMovePcx[v10]);
    v11 = sub_12F04(aButtonViewPcx);
    sub_1318C(&v72, (char *)aButtonViewPcx, (int)&aButtonViewPcx[v11]);
    v12 = sub_12F04(aButtonInfoPcx);
    sub_1318C(&v73, (char *)aButtonInfoPcx, (int)&aButtonInfoPcx[v12]);
    v13 = sub_12F04(aButtonCardPcx);
    sub_1318C(&v74, (char *)aButtonCardPcx, (int)&aButtonCardPcx[v13]);
    v14 = sub_12F04(aButtonSystemPc);
    sub_1318C(&v75, (char *)aButtonSystemPc, (int)&aButtonSystemPc[v14]);
    v15 = sub_12F04(aButtonLoadPcx);
    sub_1318C(&v76, (char *)aButtonLoadPcx, (int)&aButtonLoadPcx[v15]);
    v16 = sub_12F04(aButtonSavePcx);
    sub_1318C(&v77, (char *)aButtonSavePcx, (int)&aButtonSavePcx[v16]);
    v17 = sub_12F04(aButtonOptionPc);
    sub_1318C(&v78, (char *)aButtonOptionPc, (int)&aButtonOptionPc[v17]);
    v18 = sub_12F04(aButtonExitPcx);
    sub_1318C(&v79, (char *)aButtonExitPcx, (int)&aButtonExitPcx[v18]);
    v19 = sub_12F04(aButtonReturnPc);
    sub_1318C(&v80, (char *)aButtonReturnPc, (int)&aButtonReturnPc[v19]);
    v20 = sub_12F04(aButtonBankPcx);
    sub_1318C(&v81, (char *)aButtonBankPcx, (int)&aButtonBankPcx[v20]);
    v21 = sub_12F04(aButtonInvestPc);
    sub_1318C(&v82, (char *)aButtonInvestPc, (int)&aButtonInvestPc[v21]);
    v22 = sub_12F04(aButtonTroopPcx);
    sub_1318C(&v83, (char *)aButtonTroopPcx, (int)&aButtonTroopPcx[v22]);
    v23 = sub_12F04(aButtonAimlandP);
    sub_1318C(&v84, (char *)aButtonAimlandP, (int)&aButtonAimlandP[v23]);
    v24 = sub_12F04(aButtonAimroleP);
    sub_1318C(&v85, (char *)aButtonAimroleP, (int)&aButtonAimroleP[v24]);
    v25 = sub_12F04(aButtonAimcastl);
    sub_1318C(&v86, (char *)aButtonAimcastl, (int)&aButtonAimcastl[v25]);
    v26 = sub_12F04(aButtonCaptureP);
    sub_1318C(&v87, (char *)aButtonCaptureP, (int)&aButtonCaptureP[v26]);
    v27 = sub_12F04(aButtonNotcaptu);
    sub_1318C(&v88, (char *)aButtonNotcaptu, (int)&aButtonNotcaptu[v27]);
    v28 = sub_12F04(aButtonAssignPc);
    sub_1318C(&v89, (char *)aButtonAssignPc, (int)&aButtonAssignPc[v28]);
    v29 = sub_12F04(aButtonBuildPcx);
    sub_1318C(&v90, (char *)aButtonBuildPcx, (int)&aButtonBuildPcx[v29]);
    v30 = sub_12F04(aButtonTradePcx);
    sub_1318C(&v91, (char *)aButtonTradePcx, (int)&aButtonTradePcx[v30]);
    v31 = sub_12F04(aButtonSellPcx);
    sub_1318C(&v92, (char *)aButtonSellPcx, (int)&aButtonSellPcx[v31]);
    v32 = sub_12F04(aButtonBuyPcx);
    sub_1318C(&v93, (char *)aButtonBuyPcx, (int)&aButtonBuyPcx[v32]);
    v33 = sub_12F04(aButtonPayPcx);
    sub_1318C(&v94, (char *)aButtonPayPcx, (int)&aButtonPayPcx[v33]);
    v34 = sub_12F04(aButtonAttackPc);
    sub_1318C(&v95, (char *)aButtonAttackPc, (int)&aButtonAttackPc[v34]);
    v35 = sub_12F04(aButtonExploreP);
    sub_1318C(&v96, (char *)aButtonExploreP, (int)&aButtonExploreP[v35]);
    v36 = sub_12F04(aButtonFinishPc);
    sub_1318C(&v97, (char *)aButtonFinishPc, (int)&aButtonFinishPc[v36]);
    v37 = sub_12F04(aButtonBasePcx);
    sub_1318C((void **)&v98, (char *)aButtonBasePcx, (int)&aButtonBasePcx[v37]);
    v64 = v70[0];
    v38 = sub_1E5CC(&v64);
    v39 = sub_1E84C((int)v68, v38);
    v40 = v39;
    if ( v39 == 0 )
      __debugbreak();
    v41 = operator new(*(_DWORD *)(v39 + 4));
    v42 = v41;
    if ( v41 == nullptr )
    {
      v43 = v99;
      do
      {
        v43 -= 12;
        sub_12EA0((int)v43);
        --v6;
      }
      while ( v6 != 0 );
LABEL_27:
      v60 = &off_6C910;
      sub_1E798(v68);
      return 0;
    }
    sub_63CE8((int)v68, v41);
    v61 = 1;
    v62 = 16;
    *(_DWORD *)(a1 + 44) = sub_645B4(&v60, v42, *(_DWORD *)(v40 + 4));
    operator delete(v42);
    if ( *(_DWORD *)(a1 + 44) == 0 )
    {
      v44 = v99;
      do
      {
        v44 -= 12;
        sub_12EA0((int)v44);
        --v6;
      }
      while ( v6 != 0 );
      goto LABEL_27;
    }
    v45 = 1;
    v46 = v71;
    do
    {
      v66 = *v46;
      v47 = sub_1E5CC(&v66);
      v48 = sub_1E84C((int)v68, v47);
      v49 = v48;
      if ( v48 == 0 )
        __debugbreak();
      v50 = operator new(*(_DWORD *)(v48 + 4));
      v51 = v50;
      if ( v50 == nullptr )
      {
        v59 = v99;
        do
        {
          v59 -= 12;
          sub_12EA0((int)v59);
          --v6;
        }
        while ( v6 != 0 );
        goto LABEL_27;
      }
      sub_63CE8((int)v68, v50);
      v62 = 16;
      v61 = 1;
      v52 = sub_645B4(&v60, v51, *(_DWORD *)(v49 + 4));
      v67 = v45;
      *(_DWORD *)sub_33C30(a3, &v67) = v52;
      operator delete(v51);
      v65 = v45;
      if ( *(_DWORD *)(*(_DWORD *)sub_37F70(a3, v69, &v65) + 20) == 0 )
        goto LABEL_24;
      ++v45;
      v46 += 3;
    }
    while ( v45 < 28 );
    v63 = v98;
    v53 = sub_1E5CC(&v63);
    v54 = sub_1E84C((int)v68, v53);
    v55 = v54;
    if ( v54 == 0 )
      __debugbreak();
    v56 = operator new(*(_DWORD *)(v54 + 4));
    v57 = v56;
    if ( v56 == nullptr
      || (sub_63CE8((int)v68, v56),
          v62 = 16,
          v61 = 1,
          *(_DWORD *)(a1 + 40) = sub_645B4(&v60, v57, *(_DWORD *)(v55 + 4)),
          operator delete(v57),
          *(_DWORD *)(a1 + 40) == 0) )
    {
LABEL_24:
      sub_11634((int)v70, 12, 29, sub_12EA0);
      sub_1EA14(&v60);
      sub_1E798(v68);
      return 0;
    }
  }
  sub_11634((int)v70, 12, 29, sub_12EA0);
  sub_1EA14(&v60);
  sub_1E798(v68);
  return 1;
}

