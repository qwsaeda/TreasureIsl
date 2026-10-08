/* ea:   0x000503F0 */
/* name: sub_503F0 */
/* size: 1096 */
int __fastcall sub_503F0(int a1, int a2, int a3, int a4)
{
  int v6; // r0
  int result; // r0
  int v8; // r0
  int v9; // r2
  int v10; // r7
  _DWORD *v11; // r5
  int v12; // r2
  int v13; // r1
  int v14; // r0
  int v15; // r6
  int v16; // r8
  int v17; // r4
  int v18; // r0
  int v19; // r2
  int v20; // r0
  unsigned int v21; // r2
  unsigned int v22; // r1
  int v23; // r0
  unsigned int v24; // r0
  int v25; // r3
  unsigned int v26; // r2
  __int16 v27; // r1
  int v28; // r0
  unsigned int *v29; // r10
  int v30; // t1
  unsigned int v31; // r1
  int v32; // r0
  int v33; // t1
  int v34; // r0
  int v35; // t1
  int v36; // r0
  int v37; // t1
  int v38; // r0
  int v39; // t1
  unsigned int v40; // r2
  int v41; // [sp+4h] [bp-40h] BYREF
  _DWORD v42[14]; // [sp+8h] [bp-3Ch] BYREF
  int varg_r3; // [sp+40h] [bp-4h] BYREF

  v42[11] = a1;
  v42[12] = a2;
  v42[13] = a3;
  varg_r3 = a4;
  *(_DWORD *)(a1 + 24) = a2;
  if ( a2 == 0 )
    return 0;
  v6 = sub_53890(a2 + 52, a4);
  *(_DWORD *)(a1 + 12) = v6;
  if ( v6 == 0 )
    return 0;
  v8 = *(_DWORD *)(a1 + 24);
  *(_DWORD *)(a1 + 20) = 0;
  v9 = *(_DWORD *)(*(_DWORD *)sub_52D98(v8 + 520, v42, &varg_r3) + 20);
  *(_DWORD *)(a1 + 16) = v9;
  if ( v9 != 0 )
  {
    v10 = varg_r3;
    if ( varg_r3 <= 4 )
    {
      v11 = *(_DWORD **)(a1 + 24);
      *(_DWORD *)(a1 + 44) = *(_DWORD *)(v11[8] + 4 * varg_r3 + 40);
      *(_DWORD *)(a1 + 48) = *(_DWORD *)(v11[8] + 104);
      v12 = *(_DWORD *)(v11[8] + 4 * v10 + 60);
      *(_DWORD *)(a1 + 60) = v12;
      *(_DWORD *)(a1 + 52) = *(_DWORD *)(v11[8] + 112);
      v13 = *(_DWORD *)(v11[8] + 116);
      v14 = *(_DWORD *)(a1 + 44);
      *(_DWORD *)(a1 + 56) = v13;
      v15 = v14;
      *(_DWORD *)(a1 + 40) = v14;
      if ( v14 != 0 )
      {
        v16 = *(_DWORD *)(a1 + 48);
        if ( v16 != 0 && v12 != 0 && *(_DWORD *)(a1 + 52) != 0 && v13 != 0 )
        {
          v17 = *(_DWORD *)(a1 + 12);
          *(_DWORD *)(a1 + 64) = (int)*(unsigned __int16 *)(v17 + 20) % v11[173];
          v18 = *(unsigned __int16 *)(v17 + 20) / (int)v11[173];
          *(_DWORD *)(a1 + 8) = v10;
          *(_DWORD *)(a1 + 68) = v18;
          *(_BYTE *)(a1 + 128) = 15;
          *(_DWORD *)(a1 + 32) = 0;
          v19 = *(_DWORD *)(v11[66] + 4 * v10 - 4);
          *(_DWORD *)(a1 + 28) = 0;
          *(_DWORD *)(a1 + 36) = 32 * v19;
          if ( dword_7FB70 != 5 || (v20 = 8, v10 != 1) )
            v20 = 4;
          *(_DWORD *)(a1 + 100) = v20;
          v21 = *(_DWORD *)(a1 + 100);
          *(_DWORD *)(a1 + 104) = 3;
          *(_DWORD *)(a1 + 96) = v21;
          *(_DWORD *)(a1 + 80) = *(_DWORD *)(v15 + 4) >> 2;
          *(_DWORD *)(a1 + 84) = *(_DWORD *)(v15 + 8) / v21;
          *(_DWORD *)(a1 + 88) = *(_DWORD *)(v16 + 4) >> 2;
          v22 = *(_DWORD *)(v16 + 8);
          *(_DWORD *)(a1 + 124) = a3;
          *(_DWORD *)(a1 + 72) = *(_DWORD *)(a1 + 80);
          *(_DWORD *)(a1 + 92) = v22 / 3;
          *(_DWORD *)(a1 + 76) = *(_DWORD *)(a1 + 84);
          sub_52AD8(a1 + 188, 0xAu);
          sub_5290C(a1 + 188);
          v23 = *(_DWORD *)(a1 + 24);
          v41 = a1;
          sub_11558(v23 + 64, v42, *(_DWORD *)(v23 + 64), &v41);
          sub_3958C((_DWORD **)(*(_DWORD *)(a1 + 24) + 64));
          *(_BYTE *)(a1 + 129) = 0;
          *(_BYTE *)(a1 + 130) = -1;
          v24 = *(_DWORD *)(a1 + 144);
          *(_DWORD *)(a1 + 148) = 999999;
          if ( v24 > 0xF423F )
            *(_DWORD *)(a1 + 144) = 999999;
          sub_164D4((unsigned __int16 *)(a1 + 136), 999);
          v25 = *(_DWORD *)(a1 + 16);
          v26 = *(unsigned __int16 *)(v25 + 12);
          v27 = 999;
          if ( v26 <= 0x3E2 )
            v27 = v26 + 5;
          *(_WORD *)(v25 + 12) = v27;
          v28 = sub_186F0(*(_DWORD *)(a1 + 24), *(_DWORD *)(a1 + 8));
          if ( (unsigned __int8)sub_52D2C(v28, aAstx3t) != 0 )
          {
            sub_3053C((unsigned __int16 *)(a1 + 136), 20);
            v30 = *(_DWORD *)(a1 + 144);
            v29 = (unsigned int *)(a1 + 144);
            v31 = v30 + 30000;
            goto LABEL_28;
          }
          v32 = sub_186F0(*(_DWORD *)(a1 + 24), *(_DWORD *)(a1 + 8));
          if ( (unsigned __int8)sub_52D2C(v32, aKq) != 0 )
          {
            sub_3053C((unsigned __int16 *)(a1 + 136), 15);
            v33 = *(_DWORD *)(a1 + 144);
            v29 = (unsigned int *)(a1 + 144);
            v31 = v33 + 40000;
            goto LABEL_28;
          }
          v34 = sub_186F0(*(_DWORD *)(a1 + 24), *(_DWORD *)(a1 + 8));
          if ( (unsigned __int8)sub_52D2C(v34, asc_7DFF8) != 0 )
          {
            sub_3053C((unsigned __int16 *)(a1 + 136), 35);
            v35 = *(_DWORD *)(a1 + 144);
            v29 = (unsigned int *)(a1 + 144);
            v31 = v35 + 20000;
            goto LABEL_28;
          }
          v36 = sub_186F0(*(_DWORD *)(a1 + 24), *(_DWORD *)(a1 + 8));
          if ( (unsigned __int8)sub_52D2C(v36, aGy) != 0 )
          {
            sub_3053C((unsigned __int16 *)(a1 + 136), 50);
            sub_32410((unsigned __int16 *)(a1 + 136), 5);
            v37 = *(_DWORD *)(a1 + 144);
            v29 = (unsigned int *)(a1 + 144);
            v31 = v37 + 15000;
            goto LABEL_28;
          }
          v38 = sub_186F0(*(_DWORD *)(a1 + 24), *(_DWORD *)(a1 + 8));
          if ( (unsigned __int8)sub_52D2C(v38, byte_7DFE4) != 0 )
          {
            sub_3053C((unsigned __int16 *)(a1 + 136), 30);
            sub_32410((unsigned __int16 *)(a1 + 136), 20);
            v39 = *(_DWORD *)(a1 + 144);
            v29 = (unsigned int *)(a1 + 144);
            v31 = v39 + 50000;
LABEL_28:
            v40 = v29[1];
            result = 1;
            *v29 = v31;
            if ( v31 > v40 )
              *v29 = v40;
            return result;
          }
        }
      }
    }
  }
  return 0;
}

