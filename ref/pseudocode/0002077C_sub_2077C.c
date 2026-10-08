/* ea:   0x0002077C */
/* name: sub_2077C */
/* size: 2204 */
int __fastcall sub_2077C(int a1, int a2, int a3)
{
  void **v6; // r5
  int v7; // r0
  int v8; // r5
  int v9; // r0
  int v10; // r5
  int v11; // r0
  int v12; // r5
  int v13; // r0
  int v14; // r5
  int v15; // r0
  int v16; // r5
  int v17; // r0
  int v18; // r1
  int v19; // r7
  int v20; // r4
  int v21; // r3
  __int16 **v22; // r1
  void **v23; // r0
  int v24; // r2
  int v25; // t1
  int *v26; // r1
  int v27; // t1
  int v28; // r3
  int v29; // t1
  int v30; // r3
  int v31; // r4
  wchar_t *v32; // r1
  void *v33; // r0
  int v34; // r0
  _DWORD *v35; // r4
  _DWORD *v36; // r0
  void *v38; // r0
  int v39; // r0
  int v40; // r4
  _DWORD *v41; // r0
  int v42; // r4
  int v43; // r4
  int v44; // r1
  void *v45; // r4
  int v46; // r0
  int v47; // r3
  int i; // [sp+10h] [bp-90h] BYREF
  int v49[2]; // [sp+14h] [bp-8Ch] BYREF
  _BYTE v50[4]; // [sp+1Ch] [bp-84h] BYREF
  int v51; // [sp+20h] [bp-80h]
  _DWORD v52[4]; // [sp+24h] [bp-7Ch] BYREF
  void *v53[4]; // [sp+34h] [bp-6Ch] BYREF
  void *v54[4]; // [sp+44h] [bp-5Ch] BYREF
  void *v55[4]; // [sp+54h] [bp-4Ch] BYREF
  void *v56[4]; // [sp+64h] [bp-3Ch] BYREF
  void *v57[11]; // [sp+74h] [bp-2Ch] BYREF

  sub_1333C(v57);
  sub_1333C(v56);
  sub_1333C(v55);
  sub_1333C(v54);
  *(_DWORD *)(a1 + 4) = a2;
  if ( a2 == 0 )
    goto LABEL_53;
  *(_DWORD *)(a1 + 8) = a3;
  if ( a3 == 0 )
    goto LABEL_53;
  sub_1333C(v53);
  sub_300C4(a1 + 276);
  sub_12EA0((int)v53);
  v6 = *(void ***)(a1 + 276);
  v7 = sub_12F04(asc_7E014);
  sub_1318C(v6, asc_7E014, (int)&asc_7E014[2 * v7]);
  v8 = *(_DWORD *)(a1 + 276);
  v9 = sub_12F04(aAstx3t);
  sub_1318C((void **)(v8 + 12), aAstx3t, (int)&aAstx3t[2 * v9]);
  v10 = *(_DWORD *)(a1 + 276);
  v11 = sub_12F04(aKq);
  sub_1318C((void **)(v10 + 24), aKq, (int)&aKq[2 * v11]);
  v12 = *(_DWORD *)(a1 + 276);
  v13 = sub_12F04(asc_7DFF8);
  sub_1318C((void **)(v12 + 36), asc_7DFF8, (int)&asc_7DFF8[2 * v13]);
  v14 = *(_DWORD *)(a1 + 276);
  v15 = sub_12F04(aGy);
  sub_1318C((void **)(v14 + 48), aGy, (int)&aGy[2 * v15]);
  v16 = *(_DWORD *)(a1 + 276);
  v17 = sub_12F04(byte_7DFE4);
  sub_1318C((void **)(v16 + 60), byte_7DFE4, (int)&byte_7DFE4[2 * v17]);
  sub_1309C((int)v56);
  sub_162DC(v56, (char *)aDataGlpicsPak);
  if ( (unsigned __int8)sub_20088(a1, (int)v56[0], (_DWORD *)(a1 + 32)) == 0 )
    goto LABEL_53;
  v18 = *(_DWORD *)(*(_DWORD *)(a1 + 32) + 4);
  *(_DWORD *)(a1 + 16) = v18;
  if ( v18 == 0 )
    goto LABEL_53;
  v49[0] = 0;
  sub_30B78(a1 + 264);
  **(_DWORD **)(a1 + 264) = 0;
  v19 = 2;
  *(_DWORD *)(*(_DWORD *)(a1 + 264) + 4) = 1;
  *(_DWORD *)(*(_DWORD *)(a1 + 264) + 8) = 2;
  *(_DWORD *)(*(_DWORD *)(a1 + 264) + 12) = 3;
  *(_DWORD *)(*(_DWORD *)(a1 + 264) + 16) = 4;
  if ( dword_7FB70 != 1 )
  {
    sub_132FC(v53, (_DWORD *)(*(_DWORD *)(a1 + 276) + 12));
    v20 = dword_7FB70;
    v21 = *(_DWORD *)(a1 + 276);
    v22 = (__int16 **)(v21 + 12 * dword_7FB70);
    if ( v22 != (__int16 **)(v21 + 12) )
    {
      sub_17890(v21 + 12, *v22, *(__int16 **)(v21 + 12 * dword_7FB70 + 4));
      v20 = dword_7FB70;
    }
    v23 = (void **)(*(_DWORD *)(a1 + 276) + 12 * v20);
    if ( v53 != v23 )
    {
      sub_17890((int)v23, (__int16 *)v53[0], (__int16 *)v53[1]);
      v20 = dword_7FB70;
    }
    v24 = *(_DWORD *)(a1 + 32);
    v25 = *(_DWORD *)(v24 + 44);
    *(_DWORD *)(v24 + 44) = *(_DWORD *)(v24 + 4 * v20 + 40);
    *(_DWORD *)(*(_DWORD *)(a1 + 32) + 4 * dword_7FB70 + 40) = v25;
    v26 = *(int **)(a1 + 264);
    v27 = *v26;
    *v26 = v26[dword_7FB70 - 1];
    *(_DWORD *)(*(_DWORD *)(a1 + 264) + 4 * dword_7FB70 - 4) = v27;
    v28 = *(_DWORD *)(a1 + 32);
    v29 = *(_DWORD *)(v28 + 64);
    *(_DWORD *)(v28 + 64) = *(_DWORD *)(v28 + 4 * dword_7FB70 + 60);
    *(_DWORD *)(*(_DWORD *)(a1 + 32) + 4 * dword_7FB70 + 60) = v29;
    v30 = *(_DWORD *)(a1 + 32);
    v31 = *(_DWORD *)(v30 + 84);
    *(_DWORD *)(v30 + 84) = *(_DWORD *)(v30 + 4 * dword_7FB70 + 80);
    *(_DWORD *)(*(_DWORD *)(a1 + 32) + 4 * dword_7FB70 + 80) = v31;
    sub_12F48(v53);
  }
  sub_1309C((int)v55);
  sub_1309C((int)v54);
  switch ( dword_7FB6C )
  {
    case 1:
      sub_162DC(v55, (char *)aDataGlmapM01Pa);
      v32 = aDataGlmapm01Ma;
      break;
    case 2:
      sub_162DC(v55, (char *)aDataGlmapM02Pa);
      v32 = aDataGlmapm02Ma;
      break;
    case 3:
      sub_162DC(v55, (char *)aDataGlmapM01Pa);
      v32 = aDataGlmapm03Ma;
      break;
    default:
LABEL_53:
      sub_12F48(v54);
      sub_12F48(v55);
      sub_12F48(v56);
      sub_12F48(v57);
      return 0;
  }
  sub_162DC(v54, (char *)v32);
  if ( (unsigned __int8)sub_2FC44(a1, v55[0]) == 0 || (unsigned __int8)sub_2EA98(a1, v54[0]) == 0 )
    goto LABEL_53;
  v33 = operator new(0x3FCu);
  v34 = v33 != nullptr ? sub_43B8C(v33, *(_DWORD *)(a1 + 8)) : 0;
  *(_DWORD *)(a1 + 580) = v34;
  if ( v34 == 0 )
    goto LABEL_53;
  sub_1309C((int)v55);
  sub_162DC(v55, (char *)aDataUiPak);
  sub_1309C((int)v54);
  sub_162DC(v54, (char *)aDataMainMap);
  sub_43D54(*(_DWORD *)(a1 + 580), *(_DWORD *)(*(_DWORD *)(a1 + 4) + 16));
  v35 = sub_132FC(v53, v54);
  v36 = sub_132FC(v52, v55);
  (*(void (__fastcall **)(_DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD))(**(_DWORD **)(a1 + 580) + 4))(
    *(_DWORD *)(a1 + 580),
    *v36,
    v36[1],
    v36[2],
    *v35,
    v35[1],
    v35[2]);
  if ( (unsigned __int8)sub_334BC(a1 + 72) != 0 )
  {
    if ( (unsigned __int8)sub_4E5F8(a1 + 144, a1) != 0 )
    {
      i = 1;
      while ( 1 )
      {
        v38 = operator new(0x18u);
        v49[0] = v38 != nullptr ? sub_1B278((int)v38, i) : 0;
        sub_2FBF0(v50, &i, v49);
        if ( v51 == 0 )
          break;
        sub_37EC0(v52, v50);
        sub_2F9D8(a1 + 520, v53, v52);
        if ( ++i > 4 )
        {
          if ( (*(unsigned __int8 (__fastcall **)(int, int, int, int))(*(_DWORD *)(a1 + 288) + 48))(a1 + 288, a1, 3, 1) != 0 )
          {
            v39 = *(_DWORD *)(a1 + 724);
            v49[0] = 1;
            *(_DWORD *)(a1 + 724) = v39 + 1;
            *(_DWORD *)sub_2FA58(a1 + 532, v49) = 56 - *(_DWORD *)(*(_DWORD *)(a1 + 300) + 12);
            v49[0] = 1;
            *(_DWORD *)(sub_2FA58(a1 + 532, v49) + 4) = 94 - *(_DWORD *)(*(_DWORD *)(a1 + 300) + 16);
            i = 2;
            while ( 1 )
            {
              v49[0] = v19;
              v40 = operator new(0xD0u) != nullptr ? sub_466D0() : 0;
              *(_DWORD *)sub_1DD10((_DWORD *)(a1 + 496), v49) = v40;
              if ( *(_DWORD *)sub_1DD10((_DWORD *)(a1 + 496), v49) == 0 )
                break;
              v41 = (_DWORD *)sub_1DD10((_DWORD *)(a1 + 496), v49);
              if ( (*(unsigned __int8 (__fastcall **)(_DWORD, int, int, int))(*(_DWORD *)*v41 + 48))(
                     *v41,
                     a1,
                     3,
                     v49[0]) == 0 )
                break;
              ++*(_DWORD *)(a1 + 724);
              v42 = sub_1DD10((_DWORD *)(a1 + 496), v49);
              *(_DWORD *)sub_2FA58(a1 + 532, v49) = 56 - *(_DWORD *)(*(_DWORD *)(*(_DWORD *)v42 + 12) + 12);
              v43 = sub_1DD10((_DWORD *)(a1 + 496), v49);
              *(_DWORD *)(sub_2FA58(a1 + 532, v49) + 4) = 94 - *(_DWORD *)(*(_DWORD *)(*(_DWORD *)v43 + 12) + 16);
              v19 = i + 1;
              i = v19;
              if ( v19 > 4 )
              {
                if ( (unsigned __int8)sub_39EEC(a1 + 192) != 0
                  && (unsigned __int8)sub_53D48(a1 + 608, *(_DWORD *)(a1 + 604), aGoodsGoodsMdu) != 0 )
                {
                  memset((void *)(a1 + 544), 0, 6u);
                  v44 = *(_DWORD *)(a1 + 604);
                  *(_DWORD *)(a1 + 712) = 1;
                  *(_DWORD *)(a1 + 716) = 0;
                  if ( (unsigned __int8)sub_3CE74(a1 + 672, v44) != 0
                    && (unsigned __int8)sub_18BAC(a1 + 652, *(_DWORD *)(a1 + 604)) != 0 )
                  {
                    for ( i = 1; i <= 4; ++i )
                    {
                      *(_WORD *)sub_2FB3C(a1 + 552, &i) = 0;
                      *(_WORD *)sub_2FB3C(a1 + 564, &i) = 0;
                    }
                    v45 = operator new(0x50u);
                    if ( v45 != nullptr )
                    {
                      v46 = sub_53890(a1 + 52, 1);
                      v47 = sub_4F738(v45, a1, *(unsigned __int16 *)(v46 + 20), 4);
                    }
                    else
                    {
                      v47 = 0;
                    }
                    *(_DWORD *)(a1 + 596) = v47;
                    if ( v47 != 0 )
                    {
                      (*(void (__fastcall **)(int, _DWORD, _DWORD))(*(_DWORD *)v47 + 4))(
                        v47,
                        *(_DWORD *)(a1 + 4),
                        *(_DWORD *)(a1 + 8));
                      sub_12EA0((int)v54);
                      sub_12EA0((int)v55);
                      sub_12EA0((int)v56);
                      sub_12EA0((int)v57);
                      return 1;
                    }
                  }
                }
                goto LABEL_51;
              }
            }
          }
          break;
        }
      }
    }
LABEL_51:
    sub_12EA0((int)v54);
    sub_12EA0((int)v55);
    sub_12EA0((int)v56);
    sub_12EA0((int)v57);
    return 0;
  }
  else
  {
    sub_12F48(v54);
    sub_12EA0((int)v55);
    sub_12EA0((int)v56);
    sub_12EA0((int)v57);
    return 0;
  }
}

