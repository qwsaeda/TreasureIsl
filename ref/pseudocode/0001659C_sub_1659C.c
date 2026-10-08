/* ea:   0x0001659C */
/* name: sub_1659C */
/* size: 1172 */
_DWORD *__fastcall sub_1659C(_DWORD *a1, _DWORD *a2, int a3, unsigned __int16 *a4)
{
  int v8; // r1
  unsigned int v10; // r3
  unsigned int v11; // r2
  int v12; // r4
  _DWORD *v13; // r0
  int v14; // r3
  char v15; // r7
  unsigned int v16; // r2
  BOOL v17; // r6
  int v18; // r4
  bool v19; // r5
  _DWORD *v20; // r0
  _BYTE v22[4]; // [sp+Ch] [bp-50h] BYREF
  _BYTE v23[4]; // [sp+10h] [bp-4Ch] BYREF
  _BYTE v24[4]; // [sp+14h] [bp-48h] BYREF
  _BYTE v25[8]; // [sp+18h] [bp-44h] BYREF
  _BYTE v26[8]; // [sp+20h] [bp-3Ch] BYREF
  _BYTE v27[8]; // [sp+28h] [bp-34h] BYREF
  char v28; // [sp+30h] [bp-2Ch] BYREF

  v8 = *a1;
  if ( a3 == *(_DWORD *)(*a1 + 8) )
  {
    if ( a1[1] != 0 )
    {
      v10 = *(unsigned __int16 *)(a3 + 16);
      v11 = *a4;
      if ( v11 >= v10 )
      {
        if ( v10 < v11 )
        {
          v12 = sub_16A30(a3);
          if ( v12 == *a1 )
          {
            *a2 = *(_DWORD *)sub_17644(a1, v23, 0, a3, a4, a3);
            return a2;
          }
          else if ( *a4 >= (unsigned int)*(unsigned __int16 *)(v12 + 16) )
          {
            *a2 = *(_DWORD *)sub_17738(a1, &v28, a4);
            return a2;
          }
          else
          {
            if ( *(_DWORD *)(a3 + 12) != 0 )
              v13 = (_DWORD *)sub_17644(a1, v25, v12, v12, a4, 0);
            else
              v13 = (_DWORD *)sub_17644(a1, v24, 0, a3, a4, a3);
            *a2 = *v13;
            return a2;
          }
        }
        else
        {
          *a2 = a3;
          return a2;
        }
      }
      else
      {
        *a2 = *(_DWORD *)sub_17644(a1, v22, a3, a3, a4, 0);
        return a2;
      }
    }
    else
    {
      *a2 = *(_DWORD *)sub_17738(a1, v27, a4);
      return a2;
    }
  }
  else if ( a3 == v8 )
  {
    if ( *(unsigned __int16 *)(*(_DWORD *)(v8 + 12) + 16) >= (unsigned int)*a4 )
      *a2 = *(_DWORD *)sub_17738(a1, v26, a4);
    else
      *a2 = *(_DWORD *)sub_17644(a1, v26, 0, *(_DWORD *)(v8 + 12), a4, a3);
    return a2;
  }
  else
  {
    v14 = sub_17924(a3);
    v15 = 1;
    v16 = *a4;
    v17 = v16 < *(unsigned __int16 *)(a3 + 16);
    if ( v16 >= *(unsigned __int16 *)(a3 + 16) || v16 <= *(unsigned __int16 *)(v14 + 16) )
    {
      v18 = sub_16A30(a3);
      v19 = !v17;
      if ( v17 )
        goto LABEL_37;
      v19 = *(unsigned __int16 *)(a3 + 16) < (unsigned int)*a4;
      if ( *(unsigned __int16 *)(a3 + 16) >= (unsigned int)*a4 )
        goto LABEL_37;
      if ( v18 == *a1 )
        goto LABEL_33;
      if ( *a4 >= (unsigned int)*(unsigned __int16 *)(v18 + 16) )
        v15 = 0;
      if ( v15 != 0 )
      {
LABEL_33:
        if ( *(_DWORD *)(a3 + 12) != 0 )
          v20 = (_DWORD *)sub_17644(a1, v26, v18, v18, a4, 0);
        else
          v20 = (_DWORD *)sub_17644(a1, v26, 0, a3, a4, a3);
        *a2 = *v20;
        return a2;
      }
      else
      {
LABEL_37:
        if ( v17 == v19 )
          *a2 = a3;
        else
          *a2 = *(_DWORD *)sub_17738(a1, v26, a4);
        return a2;
      }
    }
    else
    {
      if ( *(_DWORD *)(v14 + 12) != 0 )
        *a2 = *(_DWORD *)sub_17644(a1, v26, a3, a3, a4, 0);
      else
        *a2 = *(_DWORD *)sub_17644(a1, v26, 0, v14, a4, v14);
      return a2;
    }
  }
}

