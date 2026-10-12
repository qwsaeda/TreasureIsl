/* ea:   0x00035E8C */
/* name: sub_35E8C */
/* size: 1016 */
_DWORD *__fastcall sub_35E8C(_DWORD *a1, _DWORD *a2, int a3, unsigned __int8 *a4)
{
  _DWORD *v8; // r1
  unsigned int v10; // r2
  unsigned int v11; // r1
  _DWORD *v12; // r2
  int v13; // r1
  int v14; // r1
  int v15; // r0
  unsigned int v16; // r1
  char v17; // r5
  BOOL v18; // r4
  int v19; // r1
  _DWORD *v20; // r2
  bool v21; // r3
  int v22; // r1
  _BYTE v23[4]; // [sp+8h] [bp-48h] BYREF
  _BYTE v24[4]; // [sp+Ch] [bp-44h] BYREF
  _BYTE v25[4]; // [sp+10h] [bp-40h] BYREF
  _BYTE v26[4]; // [sp+14h] [bp-3Ch] BYREF
  _BYTE v27[8]; // [sp+18h] [bp-38h] BYREF
  _BYTE v28[8]; // [sp+20h] [bp-30h] BYREF
  char v29; // [sp+28h] [bp-28h] BYREF

  v8 = (_DWORD *)*a1;
  if ( a3 == *(_DWORD *)(*a1 + 8) )
  {
    if ( a1[1] == 0 )
    {
      *a2 = *(_DWORD *)sub_38CA4(a1, v28, a4);
      return a2;
    }
    v10 = *(unsigned __int8 *)(a3 + 16);
    v11 = *a4;
    if ( v11 < v10 )
    {
      *a2 = *(_DWORD *)sub_38BBC(a1, v23, a3, a3, a4, 0);
      return a2;
    }
    if ( v10 < v11 )
    {
      v12 = sub_16A30((_DWORD *)a3);
      if ( v12 == (_DWORD *)*a1 )
      {
        *a2 = *(_DWORD *)sub_38BBC(a1, v24, 0, a3, a4, a3);
        return a2;
      }
      else if ( *a4 >= (unsigned int)*((unsigned __int8 *)v12 + 16) )
      {
        *a2 = *(_DWORD *)sub_38CA4(a1, &v29, a4);
        return a2;
      }
      else
      {
        if ( *(_DWORD *)(a3 + 12) != 0 )
          v13 = *(_DWORD *)sub_38BBC(a1, v26, v12, v12, a4, 0);
        else
          v13 = *(_DWORD *)sub_38BBC(a1, v25, 0, a3, a4, a3);
        *a2 = v13;
        return a2;
      }
    }
LABEL_37:
    *a2 = a3;
    return a2;
  }
  if ( (_DWORD *)a3 == v8 )
  {
    if ( *(unsigned __int8 *)(v8[3] + 16) >= (unsigned int)*a4 )
      v14 = *(_DWORD *)sub_38CA4(a1, v27, a4);
    else
      v14 = *(_DWORD *)sub_38BBC(a1, v27, 0, v8[3], a4, a3);
    *a2 = v14;
    return a2;
  }
  else
  {
    v15 = sub_17924((char *)a3);
    v16 = *a4;
    v17 = 1;
    v18 = v16 < *(unsigned __int8 *)(a3 + 16);
    if ( v16 >= *(unsigned __int8 *)(a3 + 16) || *(unsigned __int8 *)(v15 + 16) >= v16 )
    {
      v20 = sub_16A30((_DWORD *)a3);
      v21 = !v18;
      if ( v18 )
        goto LABEL_36;
      v21 = *(unsigned __int8 *)(a3 + 16) < (unsigned int)*a4;
      if ( *(unsigned __int8 *)(a3 + 16) >= (unsigned int)*a4 )
        goto LABEL_36;
      if ( v20 != (_DWORD *)*a1 )
      {
        if ( *a4 >= (unsigned int)*((unsigned __int8 *)v20 + 16) )
          v17 = 0;
        if ( v17 == 0 )
        {
LABEL_36:
          if ( v18 != v21 )
          {
            *a2 = *(_DWORD *)sub_38CA4(a1, v27, a4);
            return a2;
          }
          goto LABEL_37;
        }
      }
      if ( *(_DWORD *)(a3 + 12) != 0 )
        v22 = *(_DWORD *)sub_38BBC(a1, v27, v20, v20, a4, 0);
      else
        v22 = *(_DWORD *)sub_38BBC(a1, v27, 0, a3, a4, a3);
      *a2 = v22;
      return a2;
    }
    else
    {
      if ( *(_DWORD *)(v15 + 12) != 0 )
        v19 = *(_DWORD *)sub_38BBC(a1, v27, a3, a3, a4, 0);
      else
        v19 = *(_DWORD *)sub_38BBC(a1, v27, 0, v15, a4, v15);
      *a2 = v19;
      return a2;
    }
  }
}

