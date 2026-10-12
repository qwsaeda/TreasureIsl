/* ea:   0x00037600 */
/* name: sub_37600 */
/* size: 1016 */
char **__fastcall sub_37600(char **a1, char **a2, char *a3, _DWORD *a4)
{
  char *v8; // r1
  int v10; // r2
  char *v11; // r2
  char *v12; // r1
  char *v13; // r1
  int v14; // r3
  char v15; // r5
  BOOL v16; // r4
  char *v17; // r1
  char *v18; // r2
  bool v19; // r3
  char *v20; // r1
  char v21[4]; // [sp+8h] [bp-48h] BYREF
  char v22[4]; // [sp+Ch] [bp-44h] BYREF
  char v23[4]; // [sp+10h] [bp-40h] BYREF
  char v24[4]; // [sp+14h] [bp-3Ch] BYREF
  _BYTE v25[8]; // [sp+18h] [bp-38h] BYREF
  char v26[8]; // [sp+20h] [bp-30h] BYREF
  char v27; // [sp+28h] [bp-28h] BYREF

  v8 = *a1;
  if ( a3 == *((char **)*a1 + 2) )
  {
    if ( a1[1] == nullptr )
    {
      *a2 = *(char **)sub_39224(a1, v26, a4);
      return a2;
    }
    v10 = *((_DWORD *)a3 + 4);
    if ( *a4 < v10 )
    {
      *a2 = *(char **)sub_39130(a1, v21, a3, a3, a4, 0);
      return a2;
    }
    if ( v10 < *a4 )
    {
      v11 = (char *)sub_16A30(a3);
      if ( v11 == *a1 )
      {
        *a2 = *(char **)sub_39130(a1, v22, 0, a3, a4, a3);
        return a2;
      }
      else if ( *a4 >= *((_DWORD *)v11 + 4) )
      {
        *a2 = *(char **)sub_39224(a1, &v27, a4);
        return a2;
      }
      else
      {
        if ( *((_DWORD *)a3 + 3) != 0 )
          v12 = *(char **)sub_39130(a1, v24, v11, v11, a4, 0);
        else
          v12 = *(char **)sub_39130(a1, v23, 0, a3, a4, a3);
        *a2 = v12;
        return a2;
      }
    }
LABEL_37:
    *a2 = a3;
    return a2;
  }
  if ( a3 == v8 )
  {
    if ( *(_DWORD *)(*((_DWORD *)v8 + 3) + 16) >= *a4 )
      v13 = *(char **)sub_39224(a1, v25, a4);
    else
      v13 = *(char **)sub_39130(a1, v25, 0, *((_DWORD *)v8 + 3), a4, a3);
    *a2 = v13;
    return a2;
  }
  else
  {
    v14 = sub_17924(a3);
    v15 = 1;
    v16 = *a4 < *((_DWORD *)a3 + 4);
    if ( *a4 >= *((_DWORD *)a3 + 4) || *(_DWORD *)(v14 + 16) >= *a4 )
    {
      v18 = (char *)sub_16A30(a3);
      v19 = !v16;
      if ( v16 )
        goto LABEL_36;
      v19 = *((_DWORD *)a3 + 4) < *a4;
      if ( *((_DWORD *)a3 + 4) >= *a4 )
        goto LABEL_36;
      if ( v18 != *a1 )
      {
        if ( *a4 >= *((_DWORD *)v18 + 4) )
          v15 = 0;
        if ( v15 == 0 )
        {
LABEL_36:
          if ( v16 != v19 )
          {
            *a2 = *(char **)sub_39224(a1, v25, a4);
            return a2;
          }
          goto LABEL_37;
        }
      }
      if ( *((_DWORD *)a3 + 3) != 0 )
        v20 = *(char **)sub_39130(a1, v25, v18, v18, a4, 0);
      else
        v20 = *(char **)sub_39130(a1, v25, 0, a3, a4, a3);
      *a2 = v20;
      return a2;
    }
    else
    {
      if ( *(_DWORD *)(v14 + 12) != 0 )
        v17 = *(char **)sub_39130(a1, v25, a3, a3, a4, 0);
      else
        v17 = *(char **)sub_39130(a1, v25, 0, v14, a4, v14);
      *a2 = v17;
      return a2;
    }
  }
}

