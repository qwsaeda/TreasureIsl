/* ea:   0x0001A814 */
/* name: sub_1A814 */
/* size: 604 */
int __fastcall sub_1A814(_DWORD *a1, int a2, int a3)
{
  int v5; // r1
  int result; // r0
  int v7; // r1
  int v8; // r0
  int v9; // r0
  int v10; // r0
  bool v11; // zf
  int v12; // r1
  int v13; // r0
  int v14; // r0
  bool v15; // zf

  v5 = a1[262];
  if ( v5 == 0 )
  {
    result = 11;
    a1[257] = 11;
    if ( a3 != 13 )
      return result;
    result = 0;
    a1[261] = 0;
    goto LABEL_4;
  }
  result = a1[256];
  if ( result != 1 )
  {
    if ( a2 != 0 )
      return result;
    if ( a3 == 13 )
    {
      result = a1[257];
      if ( result == 1 )
      {
        a1[252] = 1;
        return 1;
      }
      else if ( result == 2 )
      {
        a1[256] = 1;
        a1[261] = 0;
        a1[257] = 1;
        return 0;
      }
    }
    else if ( a3 == 37 || a3 == 39 )
    {
      result = a1[257];
      if ( result == 1 )
      {
        a1[257] = 2;
        v14 = a1[263];
        v15 = v14 == 0;
        if ( v14 != 0 )
          *(_DWORD *)(v14 + 20) = 1;
        v12 = 21;
        v13 = a1[255];
        if ( !v15 )
          a1[263] = 0;
      }
      else
      {
        if ( result != 2 )
          return result;
        v10 = a1[263];
        a1[257] = 1;
        v11 = v10 == 0;
        if ( v10 != 0 )
          *(_DWORD *)(v10 + 20) = 1;
        v12 = 11;
        v13 = a1[255];
        if ( !v11 )
          a1[263] = 0;
      }
      result = sub_445F4(v13, v12);
      a1[263] = result;
      *(_DWORD *)(result + 20) = 0;
    }
    return result;
  }
  if ( a2 != 0 )
    return result;
  if ( a3 > 38 )
  {
    if ( a3 != 39 )
    {
      if ( a3 != 40 )
        return result;
      v9 = a1[257];
      if ( v9 == 11 )
        goto LABEL_19;
      result = v9 + 3;
      a1[257] = result;
      if ( result > v5 )
      {
        a1[257] = 11;
        return 11;
      }
      return result;
    }
    result = a1[257];
    if ( result == 11 )
    {
LABEL_29:
      if ( v5 <= 0 )
      {
        a1[257] = v5;
        return result;
      }
LABEL_19:
      a1[257] = 1;
      return 1;
    }
    a1[257] = ++result;
    if ( result <= v5 )
      return result;
LABEL_28:
    a1[257] = v5;
    return result;
  }
  if ( a3 == 38 )
  {
    result = a1[257];
    if ( result != 11 )
    {
      if ( result == 10 )
      {
        a1[257] = 8;
        return 8;
      }
      a1[257] = result - 3;
      if ( result - 3 > 0 )
        return result;
      goto LABEL_19;
    }
    goto LABEL_28;
  }
  if ( a3 != 13 )
  {
    if ( a3 != 37 )
      return result;
    result = a1[257];
    if ( result != 11 )
    {
      a1[257] = --result;
      if ( result <= 0 )
        a1[257] = v5;
      return result;
    }
    goto LABEL_29;
  }
  result = a1[257];
  if ( result == 11 )
  {
LABEL_4:
    a1[252] = 1;
    return result;
  }
  v7 = a1[258] + 36 * result;
  a1[256] = 2;
  a1[257] = 1;
  v8 = a1[255];
  a1[261] = v7 - 36;
  result = sub_445F4(v8, 11);
  a1[263] = result;
  *(_DWORD *)(result + 20) = 0;
  return result;
}

