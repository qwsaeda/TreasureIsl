/* ea:   0x00054D1C */
/* name: sub_54D1C */
/* size: 1268 */
int __fastcall sub_54D1C(int a1, int a2)
{
  int v3; // r1
  int v4; // r4
  int v5; // r0
  int v6; // r2
  int v7; // r4
  int v8; // r5
  int v9; // r0
  int v10; // r2
  int v11; // r1
  bool v12; // zf
  int v13; // r1
  int v14; // r0
  int v15; // r6
  int v16; // r5
  int v17; // r4
  int v18; // r2
  int v19; // r1
  int v20; // r3
  int v21; // r4
  int v22; // r5
  int v23; // r0
  int v24; // r2
  int v25; // r1
  bool v26; // zf
  int v27; // r1
  int v28; // r0
  int v29; // r6
  int v30; // r5
  int v31; // r4
  int v32; // r2
  int v33; // r0
  int v34; // r6
  int v35; // r4
  int v36; // r5
  int v37; // r0
  int v38; // r3
  int v39; // r1
  int result; // r0

  if ( a2 != 0 )
  {
    switch ( *(_DWORD *)(a1 + 1020) )
    {
      case 1:
        *(_DWORD *)(a1 + 1108) = sub_445F4(a1, 202);
        break;
      case 2:
        *(_DWORD *)(a1 + 1108) = sub_445F4(a1, 402);
        break;
      case 3:
        *(_DWORD *)(a1 + 1108) = sub_445F4(a1, 602);
        break;
      case 4:
        *(_DWORD *)(a1 + 1108) = sub_445F4(a1, 802);
        break;
      case 5:
        *(_DWORD *)(a1 + 1108) = sub_445F4(a1, 112);
        break;
      default:
        *(_DWORD *)(a1 + 1108) = 0;
        break;
    }
    ++*(_DWORD *)(a1 + 4 * *(_DWORD *)(a1 + 1020) + 1108);
    if ( *(_DWORD *)(a1 + 1060) != 0 )
    {
      v3 = *(_DWORD *)(a1 + 1020);
      v4 = a1 + 4 * v3;
      if ( *(_DWORD *)(v4 + 1108) > (int)*(unsigned __int16 *)(*(_DWORD *)(a1 + 1028) + 8 * v3 - 4) )
        *(_DWORD *)(v4 + 1108) = *(unsigned __int16 *)(*(_DWORD *)(a1 + 1028) + 8 * v3 - 4);
      goto LABEL_66;
    }
    v12 = !sub_54C98((_DWORD *)a1);
    v5 = *(_DWORD *)(a1 + 1020);
    if ( !v12 )
    {
      v6 = a1 + 4 * v5;
      if ( *(_DWORD *)(v6 + 1108) == 1000 )
        *(_DWORD *)(v6 + 1108) = 999;
      if ( sub_55210(a1) == 0 )
      {
        v7 = 0;
        v8 = *(unsigned __int16 *)(a1 + 1052);
        if ( *(int *)(a1 + 1064) > 0 )
        {
          v9 = *(_DWORD *)(a1 + 1020);
          v10 = a1;
          v11 = v9 - 1;
          do
          {
            v12 = v7 == v11;
            if ( v7 != v11 )
              v9 = *(_DWORD *)(v10 + 1112);
            ++v7;
            v10 += 4;
            if ( !v12 )
              v8 -= v9;
          }
          while ( v7 < *(_DWORD *)(a1 + 1064) );
        }
        v13 = a1 + 4 * *(_DWORD *)(a1 + 1020);
        if ( v8 <= 0 )
          *(_DWORD *)(v13 + 1108) = 0;
        else
          *(_DWORD *)(v13 + 1108) = v8;
        if ( sub_55210(a1) == 0 )
        {
          v14 = *(_DWORD *)(a1 + 1064);
          v15 = 0;
          *(_DWORD *)(a1 + 1132) = 0;
          if ( v14 > 0 )
          {
            v16 = a1;
            v17 = *(_DWORD *)(a1 + 1020) - 1;
            do
            {
              if ( v15 != v17 )
                *(_DWORD *)(a1 + 1132) += *(_DWORD *)(*(_DWORD *)(a1 + 1040) + 8 * v15) * *(_DWORD *)(v16 + 1112);
              ++v15;
              v16 += 4;
            }
            while ( v15 < *(_DWORD *)(a1 + 1064) );
          }
          *(_DWORD *)(a1 + 4 * *(_DWORD *)(a1 + 1020) + 1108) = (unsigned int)(*(_DWORD *)(a1 + 1024)
                                                                             - *(_DWORD *)(a1 + 1132))
                                                              / *(_DWORD *)(*(_DWORD *)(a1 + 1040)
                                                                          + 8 * *(_DWORD *)(a1 + 1020)
                                                                          - 8);
          v18 = a1 + 4 * *(_DWORD *)(a1 + 1020);
          if ( *(int *)(v18 + 1108) < 0 )
            *(_DWORD *)(v18 + 1108) = 0;
        }
      }
      goto LABEL_66;
    }
LABEL_65:
    *(_DWORD *)(a1 + 4 * v5 + 1108) = 0;
    goto LABEL_66;
  }
  switch ( *(_DWORD *)(a1 + 1020) )
  {
    case 1:
      *(_DWORD *)(a1 + 1108) = sub_445F4(a1, 102);
      break;
    case 2:
      *(_DWORD *)(a1 + 1108) = sub_445F4(a1, 302);
      break;
    case 3:
      *(_DWORD *)(a1 + 1108) = sub_445F4(a1, 502);
      break;
    case 4:
      *(_DWORD *)(a1 + 1108) = sub_445F4(a1, 702);
      break;
    case 5:
      *(_DWORD *)(a1 + 1108) = sub_445F4(a1, 902);
      break;
    default:
      *(_DWORD *)(a1 + 1108) = 0;
      break;
  }
  --*(_DWORD *)(a1 + 4 * *(_DWORD *)(a1 + 1020) + 1108);
  v19 = *(_DWORD *)(a1 + 1020);
  v20 = a1 + 4 * v19;
  if ( *(int *)(v20 + 1108) < 0 )
  {
    if ( *(_DWORD *)(a1 + 1060) != 0 )
    {
      *(_DWORD *)(v20 + 1108) = *(unsigned __int16 *)(*(_DWORD *)(a1 + 1028) + 8 * v19 - 4);
      goto LABEL_66;
    }
    *(_DWORD *)(v20 + 1108) = 999;
    if ( !sub_54C98((_DWORD *)a1) )
    {
      v5 = *(_DWORD *)(a1 + 1020);
      goto LABEL_65;
    }
    if ( sub_55210(a1) == 0 )
    {
      v21 = 0;
      v22 = *(unsigned __int16 *)(a1 + 1052);
      if ( *(int *)(a1 + 1064) > 0 )
      {
        v23 = *(_DWORD *)(a1 + 1020);
        v24 = a1;
        v25 = v23 - 1;
        do
        {
          v26 = v21 == v25;
          if ( v21 != v25 )
            v23 = *(_DWORD *)(v24 + 1112);
          ++v21;
          v24 += 4;
          if ( !v26 )
            v22 -= v23;
        }
        while ( v21 < *(_DWORD *)(a1 + 1064) );
      }
      v27 = a1 + 4 * *(_DWORD *)(a1 + 1020);
      if ( v22 <= 0 )
        *(_DWORD *)(v27 + 1108) = 0;
      else
        *(_DWORD *)(v27 + 1108) = v22;
      if ( sub_55210(a1) == 0 )
      {
        v28 = *(_DWORD *)(a1 + 1064);
        v29 = 0;
        *(_DWORD *)(a1 + 1132) = 0;
        if ( v28 > 0 )
        {
          v30 = a1;
          v31 = *(_DWORD *)(a1 + 1020) - 1;
          do
          {
            if ( v29 != v31 )
              *(_DWORD *)(a1 + 1132) += *(_DWORD *)(*(_DWORD *)(a1 + 1040) + 8 * v29) * *(_DWORD *)(v30 + 1112);
            ++v29;
            v30 += 4;
          }
          while ( v29 < *(_DWORD *)(a1 + 1064) );
        }
        *(_DWORD *)(a1 + 4 * *(_DWORD *)(a1 + 1020) + 1108) = (unsigned int)(*(_DWORD *)(a1 + 1024)
                                                                           - *(_DWORD *)(a1 + 1132))
                                                            / *(_DWORD *)(*(_DWORD *)(a1 + 1040)
                                                                        + 8 * *(_DWORD *)(a1 + 1020)
                                                                        - 8);
        v32 = a1 + 4 * *(_DWORD *)(a1 + 1020);
        if ( *(int *)(v32 + 1108) < 0 )
          *(_DWORD *)(v32 + 1108) = 0;
      }
    }
  }
LABEL_66:
  v33 = *(_DWORD *)(a1 + 1064);
  v34 = 0;
  *(_DWORD *)(a1 + 1132) = 0;
  *(_DWORD *)(a1 + 1136) = 0;
  if ( v33 > 0 )
  {
    v35 = *(_DWORD *)(a1 + 1060);
    v36 = a1;
    do
    {
      if ( v35 != 0 )
        v37 = *(_DWORD *)(a1 + 1028);
      else
        v37 = *(_DWORD *)(a1 + 1040);
      *(_DWORD *)(a1 + 1136) += *(_DWORD *)(v36 + 1112);
      v38 = *(_DWORD *)(v37 + 8 * v34++);
      v39 = *(_DWORD *)(v36 + 1112);
      v36 += 4;
      *(_DWORD *)(a1 + 1132) += v38 * v39;
    }
    while ( v34 < *(_DWORD *)(a1 + 1064) );
  }
  result = *(_DWORD *)(a1 + 1108);
  if ( result != 0 )
    *(_DWORD *)(result + 20) = 0;
  return result;
}

