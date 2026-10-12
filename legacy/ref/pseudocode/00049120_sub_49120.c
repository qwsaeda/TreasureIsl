/* ea:   0x00049120 */
/* name: sub_49120 */
/* size: 1832 */
int __fastcall sub_49120(_DWORD *a1, _DWORD *a2, int a3)
{
  int v4; // r1
  int result; // r0
  unsigned __int8 v6; // r9
  int v7; // r11
  __int16 *v8; // r4
  _WORD *v9; // r3
  __int16 v10; // t1
  int v11; // r2
  int v12; // t1
  _DWORD *v13; // r4
  int v14; // r8
  _DWORD *v15; // r6
  char v16; // r0
  char *v17; // r5
  int v18; // r0
  int v19; // r4
  int v20; // r0
  _DWORD *v21; // r9
  unsigned __int16 v22; // r4
  _WORD *v23; // r0
  __int16 *v24; // r4
  int v25; // lr
  _WORD *v26; // r3
  __int16 v27; // t1
  __int16 v28; // t1
  _WORD *v29; // r5
  wchar_t *v30; // r4
  _WORD *v31; // r3
  __int16 v32; // t1
  __int16 v33; // t1
  __int16 *v34; // r4
  _WORD *v35; // r3
  __int16 v36; // t1
  __int16 v37; // t1
  _DWORD *v38; // r0
  _DWORD *v39; // r0
  unsigned __int16 *v40; // r7
  int v41; // r6
  _WORD *v42; // r5
  __int16 *v43; // r4
  _WORD *v44; // r3
  __int16 v45; // t1
  __int16 v46; // t1
  __int16 *v47; // r4
  _WORD *v48; // r3
  __int16 v49; // t1
  __int16 v50; // t1
  int v51; // r5
  int v52; // r11
  int v53; // r4
  int v54; // r0
  const void *v55; // r1
  unsigned __int8 *v56; // r10
  unsigned __int8 *v57; // r3
  unsigned int v58; // r9
  int i; // r8
  int v60; // r7
  int v61; // r6
  int v62; // r2
  int v63; // r10
  unsigned int v64; // r4
  int v65; // r5
  int v66; // r6
  int v67; // r7
  _DWORD *v68; // r8
  unsigned int v69; // r0
  unsigned int v70; // r4
  int v71; // r5
  int *v72; // r9
  unsigned int v73; // r0
  _DWORD *v74; // r2
  __int16 *v75; // r4
  int v76; // r6
  _WORD *v77; // r3
  int v78; // lr
  __int16 v79; // t1
  __int16 v80; // t1
  _WORD *v81; // r5
  wchar_t *v82; // r4
  _WORD *v83; // r3
  __int16 v84; // t1
  __int16 v85; // t1
  __int16 *v86; // r4
  _WORD *v87; // r3
  __int16 v88; // t1
  __int16 v89; // t1
  char v90; // r4
  char *v91; // r3
  __int16 v92; // t1
  __int16 v93; // t1
  char v94; // [sp+4h] [bp-98h] BYREF
  char v95[3]; // [sp+5h] [bp-97h] BYREF
  _WORD *v96; // [sp+8h] [bp-94h]
  _DWORD *v97; // [sp+Ch] [bp-90h]
  int v98; // [sp+10h] [bp-8Ch]
  int v99; // [sp+14h] [bp-88h]
  int v100; // [sp+18h] [bp-84h]
  int v101; // [sp+1Ch] [bp-80h]
  _DWORD *v102; // [sp+20h] [bp-7Ch]
  int v103; // [sp+24h] [bp-78h]
  int v104; // [sp+28h] [bp-74h] BYREF
  _BYTE v105[6]; // [sp+2Ch] [bp-70h] BYREF
  unsigned __int8 *v106; // [sp+34h] [bp-68h] BYREF
  unsigned __int8 *v107; // [sp+38h] [bp-64h]
  wchar_t v108[2]; // [sp+44h] [bp-58h] BYREF
  int v109; // [sp+48h] [bp-54h]
  int v110; // [sp+4Ch] [bp-50h]
  int v111; // [sp+50h] [bp-4Ch]
  _BYTE v112[72]; // [sp+54h] [bp-48h] BYREF

  v97 = a1;
  v4 = a1[5];
  if ( v4 == 0 )
    return 0;
  v98 = 0;
  v6 = 0;
  v103 = 0;
  v7 = a1[6] + 608;
  v100 = v7;
  v99 = *(unsigned __int8 *)(v4 + 48);
  v8 = &word_7FB48;
  v96 = (_WORD *)(a3 - 2);
  v9 = (_WORD *)(a3 - 2);
  do
  {
    v10 = v9[1];
    ++v9;
  }
  while ( v10 != 0 );
  do
  {
    v12 = (unsigned __int16)*v8++;
    v11 = v12;
    *v9++ = v12;
  }
  while ( (_WORD)v12 != 0 );
  v13 = a1 + 38;
  v102 = a1 + 38;
  v14 = 0;
  v15 = *(_DWORD **)sub_4A310(a1 + 38, &v104, v11, v9);
  while ( 1 )
  {
    v16 = v15 == *(_DWORD **)sub_4A328(v13, &v104) ? v98 : 1;
    if ( v16 == 0 )
      break;
    v17 = (char *)(v15 + 4);
    v18 = sub_3B874(v7, *((unsigned __int8 *)v15 + 16));
    v19 = v99;
    if ( (_BYTE)v99 != 0 && *(_DWORD *)(*(_DWORD *)(v18 + 24) + 4 * (unsigned __int8)v99 - 4) == 0x10000
      || rand() % 100 + 1 > 40 )
    {
      v39 = sub_16A30(v15);
      v13 = v102;
      v15 = v39;
    }
    else
    {
      v20 = sub_53BC4(v7, (unsigned __int8)*v17, v19, *((unsigned __int16 *)v15 + 9));
      v21 = v102;
      v14 += v20;
      v22 = *((_WORD *)v15 + 9);
      v94 = *v17;
      v23 = (_WORD *)sub_308D4(v102, &v94);
      sub_357C0(v23, v22);
      sub_16F5C((int)v21);
      v24 = *(__int16 **)(sub_3B874(v7, (unsigned __int8)*v17) + 4);
      v25 = v98;
      v26 = v96;
      do
      {
        v27 = v26[1];
        ++v26;
      }
      while ( v27 != 0 );
      do
      {
        v28 = *v24++;
        *v26++ = v28;
      }
      while ( v28 != 0 );
      *(_DWORD *)v108 = v25;
      v109 = v25;
      v110 = v25;
      v111 = v25;
      itow(*((unsigned __int16 *)v15 + 9), v108, 10);
      v29 = v96;
      v30 = v108;
      v31 = v96;
      do
      {
        v32 = v31[1];
        ++v31;
      }
      while ( v32 != 0 );
      do
      {
        v33 = *v30++;
        *v31++ = v33;
      }
      while ( v33 != 0 );
      v34 = &word_7FB40;
      v35 = v29;
      do
      {
        v36 = v35[1];
        ++v35;
      }
      while ( v36 != 0 );
      do
      {
        v37 = *v34++;
        *v35++ = v37;
      }
      while ( v37 != 0 );
      v6 = 1;
      v38 = sub_16A30(v15);
      v13 = v102;
      v15 = v38;
    }
  }
  v40 = (unsigned __int16 *)v97;
  v101 = v6;
  if ( v6 != 0 )
  {
    sub_1654C(v97 + 36, v14);
    v41 = v100;
    v42 = v96;
  }
  else
  {
    v42 = v96;
    v43 = &word_7FB38;
    v41 = v100;
    v44 = v96;
    do
    {
      v45 = v44[1];
      ++v44;
    }
    while ( v45 != 0 );
    do
    {
      v46 = *v43++;
      *v44++ = v46;
    }
    while ( v46 != 0 );
  }
  v47 = &word_7FB2C;
  v48 = v42;
  do
  {
    v49 = v48[1];
    ++v48;
  }
  while ( v49 != 0 );
  do
  {
    v50 = *v47++;
    *v48++ = v50;
  }
  while ( v50 != 0 );
  v51 = sub_16C9C(v40 + 68, 0);
  v104 = v51;
  v52 = sub_16C9C(v40 + 68, 3);
  if ( v51 <= v52 )
  {
    v76 = v101;
    v81 = v96;
    v90 = v103;
  }
  else
  {
    sub_4A338(&v106, v95);
    if ( *((_DWORD *)v40 + 39) < 5u )
    {
      sub_539E4(v41, v99, *(unsigned __int8 *)(*((_DWORD *)v40 + 5) + 49), &v106);
      sub_4AEA0(v112, 0);
      v53 = sub_16C9C(v40 + 68, 2);
      v54 = a2[7];
      if ( v54 != 0 )
      {
        if ( v54 != 1 )
          goto LABEL_40;
        v55 = (char *)a2 + 22;
      }
      else
      {
        v55 = a2 + 4;
      }
      memcpy(v105, v55, sizeof(v105));
LABEL_40:
      v56 = v106;
      v57 = v107;
      v58 = (unsigned int)v105[0] * v53 / 0x14;
      for ( i = (v107 - v106) / 36; i > 1; v57 = v107 )
      {
        v60 = i - 1;
        v61 = v98;
        do
        {
          v62 = v56[v61];
          v63 = v100;
          v64 = sub_53A4C(v100, v97, v62, v99, 1);
          v65 = v61 + 36;
          if ( v64 < sub_53A4C(v63, v97, v106[v61 + 36], v99, 1) )
          {
            sub_49848(v112, &v106[v61]);
            sub_49848(&v106[v61], &v106[v65]);
            sub_49848(&v106[v65], v112);
          }
          --v60;
          v56 = v106;
          v61 += 36;
        }
        while ( v60 != 0 );
        --i;
      }
      v66 = v100;
      v67 = v99;
      v68 = v97;
      while ( 1 )
      {
        if ( v56 == v57 )
        {
          v76 = v101;
          v81 = v96;
          v90 = v103;
          sub_39E18((int)v112);
          sub_2F710((int *)&v106);
          goto LABEL_70;
        }
        v69 = sub_53A4C(v66, v68, *v56, v67, 1);
        v70 = v69;
        if ( v69 != 0 )
        {
          v71 = v58 / v69;
          if ( (int)(v58 / v69) > 0 && rand() % 100 + 1 <= 50 )
            break;
        }
        v57 = v107;
        v56 += 36;
      }
      v72 = v102;
      if ( v71 > v104 - v52 )
        v71 = v104 - v52;
      v74 = v97 + 36;
      v73 = v97[36];
      v97 += 36;
      if ( v71 * v70 < v73 )
        *v97 = v73 - v71 * v70;
      else
        *v74 = v98;
      sub_30810(v72, *v56, (unsigned __int16)v71);
      v75 = *((__int16 **)v56 + 1);
      v76 = v101;
      v77 = v96;
      v78 = v98;
      do
      {
        v79 = v77[1];
        ++v77;
      }
      while ( v79 != 0 );
      do
      {
        v80 = *v75++;
        *v77++ = v80;
      }
      while ( v80 != 0 );
      *(_DWORD *)v108 = v78;
      v109 = v78;
      v110 = v78;
      v111 = v78;
      itow(v71, v108, 10);
      v81 = v96;
      v82 = v108;
      v83 = v96;
      do
      {
        v84 = v83[1];
        ++v83;
      }
      while ( v84 != 0 );
      do
      {
        v85 = *v82++;
        *v83++ = v85;
      }
      while ( v85 != 0 );
      v86 = &word_7FB28;
      v87 = v81;
      do
      {
        v88 = v87[1];
        ++v87;
      }
      while ( v88 != 0 );
      do
      {
        v89 = *v86++;
        *v87++ = v89;
      }
      while ( v89 != 0 );
      v90 = 1;
      sub_39E18((int)v112);
      sub_2F710((int *)&v106);
      goto LABEL_70;
    }
    v76 = v101;
    v81 = v96;
    v90 = v103;
    sub_2F710((int *)&v106);
  }
LABEL_70:
  if ( v90 == 0 )
  {
    v91 = asc_7FB24;
    do
    {
      v92 = v81[1];
      ++v81;
    }
    while ( v92 != 0 );
    do
    {
      v93 = *(_WORD *)v91;
      v91 += 2;
      *v81++ = v93;
    }
    while ( v93 != 0 );
  }
  if ( v76 != 0 || (LOBYTE(result) = v98, v90 != 0) )
    LOBYTE(result) = 1;
  return (unsigned __int8)result;
}

