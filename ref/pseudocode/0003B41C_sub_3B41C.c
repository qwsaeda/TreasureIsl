/* ea:   0x0003B41C */
/* name: sub_3B41C */
/* size: 532 */
int __fastcall sub_3B41C(int result, _BYTE *a2, unsigned __int16 *a3, int a4)
{
  unsigned __int16 *v5; // r9
  int v7; // r3
  int v8; // r8
  int v9; // r7
  int v10; // r5
  int v11; // r6
  int v12; // r1
  int v13; // r5
  int v14; // r4
  int *v15; // r6
  _WORD *v16; // r2
  char v17; // r0
  int v18; // r1
  int v19; // r0
  unsigned int v20; // r2
  bool v21; // cf
  int *v22; // r0
  int *v23; // r8
  int v24; // r6
  _WORD *v25; // r5
  _WORD *v26; // r0
  _WORD *v27; // r0
  _WORD *v28; // r4
  char v29[4]; // [sp+0h] [bp-34h] BYREF
  int *v30; // [sp+4h] [bp-30h]
  int v31; // [sp+8h] [bp-2Ch] BYREF
  unsigned int v32; // [sp+Ch] [bp-28h] BYREF

  v30 = (int *)result;
  v5 = a3;
  if ( a3 != (unsigned __int16 *)a4 )
  {
    v7 = *(_DWORD *)(result + 4);
    v8 = (a4 - (int)a3) >> 1;
    if ( v8 + 1 > (*(_DWORD *)(result + 8) - v7) >> 1 )
    {
      v19 = *(_DWORD *)result;
      v31 = (a4 - (int)a3) >> 1;
      v20 = (v7 - v19) >> 1;
      v21 = v20 >= v8;
      v32 = v20;
      v22 = &v31;
      v23 = v30;
      if ( v21 )
        v22 = (int *)&v32;
      v24 = *v22 + v20 + 1;
      v25 = sub_1343C((int)(v30 + 2), v24);
      v26 = sub_13898(*v30, (int)a2, v25);
      v27 = sub_13898((int)v5, a4, v26);
      v28 = sub_13898((int)a2, v30[1], v27);
      *v28 = 0;
      if ( *v23 != 0 )
        free((void *)*v23);
      *v23 = (int)v25;
      result = (int)&v25[v24];
      v23[1] = (int)v28;
      v23[2] = result;
    }
    else
    {
      v9 = (v7 - (int)a2) >> 1;
      v10 = result;
      v11 = *(_DWORD *)(result + 4);
      if ( v9 < v8 )
      {
        v13 = (int)&a3[v9 + 1];
        v14 = v11 + 2;
        sub_13898(v13, a4, (_WORD *)(v11 + 2));
        v15 = v30;
        v16 = (_WORD *)(v30[1] + 2 * (v8 - v9));
        v17 = v29[0];
        v30[1] = (int)v16;
        v29[0] = v17;
        sub_13898((int)a2, v14, v16);
        result = v15[1] + 2 * v9;
        v15[1] = result;
        if ( v5 != (unsigned __int16 *)v13 )
        {
          v18 = a2 - (_BYTE *)v5;
          do
          {
            result = *v5;
            *(unsigned __int16 *)((char *)v5++ + v18) = result;
          }
          while ( v5 != (unsigned __int16 *)v13 );
        }
      }
      else
      {
        sub_3B630(v11 - 2 * v8 + 2, v11 + 2, v11 + 2, v29);
        *(_DWORD *)(v10 + 4) += 2 * v8;
        if ( v9 - v8 != -1 )
          memmove(&a2[2 * v8], a2, 2 * (v9 - v8 + 1));
        v12 = a2 - (_BYTE *)v5;
        do
        {
          result = *v5;
          *(unsigned __int16 *)((char *)v5++ + v12) = result;
        }
        while ( v5 != (unsigned __int16 *)a4 );
      }
    }
  }
  return result;
}

