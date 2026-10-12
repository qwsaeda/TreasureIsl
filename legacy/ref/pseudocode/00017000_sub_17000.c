/* ea:   0x00017000 */
/* name: sub_17000 */
/* size: 964 */
char *__fastcall sub_17000(char *a1, char **a2, char **a3, char **a4)
{
  char *v6; // r8
  char *v7; // r1
  char *v8; // r6
  char *v9; // r5
  int v10; // r0
  char *k; // r0
  char v12; // r1
  int v13; // r1
  char v14; // r1
  int v15; // r1
  char *v16; // r0
  char *i; // r1
  char *v18; // r0
  char *j; // r1
  int v20; // r2
  _BYTE *v21; // r0
  _BYTE *v22; // r0
  _BYTE *v23; // r0
  _BYTE *v24; // r0
  _BYTE *v25; // r0
  _BYTE *v26; // r0
  _BYTE *v27; // r0
  _BYTE *v28; // r0
  _BYTE *v29; // r0
  _BYTE *v30; // r0
  char *result; // r0

  v6 = a1;
  v7 = *((char **)a1 + 2);
  if ( v7 == nullptr )
  {
    v8 = *((char **)a1 + 3);
LABEL_3:
    v9 = *((char **)v6 + 1);
    if ( v8 != nullptr )
      *((_DWORD *)v8 + 1) = v9;
    if ( *a2 == a1 )
    {
      *a2 = v8;
    }
    else
    {
      v15 = *((_DWORD *)a1 + 1);
      if ( *(char **)(v15 + 8) == a1 )
        *(_DWORD *)(v15 + 8) = v8;
      else
        *(_DWORD *)(v15 + 12) = v8;
    }
    if ( *a3 == a1 )
    {
      if ( *((_DWORD *)a1 + 3) != 0 )
      {
        v16 = *((char **)v8 + 2);
        for ( i = v8; v16 != nullptr; v16 = *((char **)v16 + 2) )
          i = v16;
        *a3 = i;
      }
      else
      {
        *a3 = *((char **)a1 + 1);
      }
    }
    if ( *a4 == a1 )
    {
      if ( *((_DWORD *)a1 + 2) != 0 )
      {
        v18 = *((char **)v8 + 3);
        for ( j = v8; v18 != nullptr; v18 = *((char **)v18 + 3) )
          j = v18;
        *a4 = j;
      }
      else
      {
        *a4 = *((char **)a1 + 1);
      }
    }
    goto LABEL_38;
  }
  v10 = *((_DWORD *)a1 + 3);
  if ( v10 == 0 )
  {
    v8 = v7;
    goto LABEL_3;
  }
  v6 = *((char **)a1 + 3);
  for ( k = *(char **)(v10 + 8); k != nullptr; k = *((char **)k + 2) )
    v6 = k;
  v8 = *((char **)v6 + 3);
  if ( v6 == a1 )
    goto LABEL_3;
  *((_DWORD *)v7 + 1) = v6;
  *((_DWORD *)v6 + 2) = *((_DWORD *)a1 + 2);
  if ( v6 == *((char **)a1 + 3) )
  {
    v9 = v6;
  }
  else
  {
    v9 = *((char **)v6 + 1);
    if ( v8 != nullptr )
      *((_DWORD *)v8 + 1) = v9;
    *(_DWORD *)(*((_DWORD *)v6 + 1) + 8) = v8;
    *((_DWORD *)v6 + 3) = *((_DWORD *)a1 + 3);
    *(_DWORD *)(*((_DWORD *)a1 + 3) + 4) = v6;
  }
  if ( *a2 == a1 )
  {
    *a2 = v6;
    v12 = *v6;
    *((_DWORD *)v6 + 1) = *((_DWORD *)a1 + 1);
    *v6 = *a1;
    v6 = a1;
    *a1 = v12;
  }
  else
  {
    v13 = *((_DWORD *)a1 + 1);
    if ( *(char **)(v13 + 8) == a1 )
      *(_DWORD *)(v13 + 8) = v6;
    else
      *(_DWORD *)(v13 + 12) = v6;
    v14 = *v6;
    *((_DWORD *)v6 + 1) = *((_DWORD *)a1 + 1);
    *v6 = *a1;
    v6 = a1;
    *a1 = v14;
  }
LABEL_38:
  if ( *v6 == 0 )
    return v6;
  if ( v8 != *a2 )
  {
    while ( v8 == nullptr || *v8 == 1 )
    {
      v20 = *((_DWORD *)v9 + 2);
      if ( v8 == (char *)v20 )
      {
        v20 = *((_DWORD *)v9 + 3);
        if ( *(_BYTE *)v20 == 0 )
        {
          *(_BYTE *)v20 = 1;
          *v9 = 0;
          sub_179A0(v9, a2);
          v20 = *((_DWORD *)v9 + 3);
        }
        v21 = *(_BYTE **)(v20 + 8);
        if ( v21 != nullptr && *v21 != 1 || (v22 = *(_BYTE **)(v20 + 12)) != nullptr && *v22 != 1 )
        {
          v23 = *(_BYTE **)(v20 + 12);
          if ( v23 == nullptr || *v23 == 1 )
          {
            v24 = *(_BYTE **)(v20 + 8);
            if ( v24 != nullptr )
              *v24 = 1;
            *(_BYTE *)v20 = 0;
            sub_17A18(v20, a2);
            v20 = *((_DWORD *)v9 + 3);
          }
          *(_BYTE *)v20 = *v9;
          *v9 = 1;
          v25 = *(_BYTE **)(v20 + 12);
          if ( v25 != nullptr )
            *v25 = 1;
          sub_179A0(v9, a2);
          break;
        }
      }
      else
      {
        if ( *(_BYTE *)v20 == 0 )
        {
          *(_BYTE *)v20 = 1;
          *v9 = 0;
          sub_17A18(v9, a2);
          v20 = *((_DWORD *)v9 + 2);
        }
        v26 = *(_BYTE **)(v20 + 12);
        if ( v26 != nullptr && *v26 != 1 || (v27 = *(_BYTE **)(v20 + 8)) != nullptr && *v27 != 1 )
        {
          v28 = *(_BYTE **)(v20 + 8);
          if ( v28 == nullptr || *v28 == 1 )
          {
            v29 = *(_BYTE **)(v20 + 12);
            if ( v29 != nullptr )
              *v29 = 1;
            *(_BYTE *)v20 = 0;
            sub_179A0(v20, a2);
            v20 = *((_DWORD *)v9 + 2);
          }
          *(_BYTE *)v20 = *v9;
          *v9 = 1;
          v30 = *(_BYTE **)(v20 + 8);
          if ( v30 != nullptr )
            *v30 = 1;
          sub_17A18(v9, a2);
          break;
        }
      }
      *(_BYTE *)v20 = 0;
      v8 = v9;
      v9 = *((char **)v9 + 1);
      if ( v8 == *a2 )
        break;
    }
  }
  result = v6;
  if ( v8 != nullptr )
    *v8 = 1;
  return result;
}

