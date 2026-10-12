/* ea:   0x00018704 */
/* name: sub_18704 */
/* size: 996 */
unsigned int __fastcall sub_18704(int a1, _BYTE *a2, int a3)
{
  unsigned int result; // r0
  unsigned int v7; // r1
  _DWORD *v8; // r4
  unsigned __int8 v9; // r7
  _DWORD *v10; // r4
  unsigned __int8 v11; // r8
  _DWORD *v12; // r5
  unsigned __int8 v13; // r8
  int v14; // r2
  unsigned __int8 v15; // r8
  _DWORD *i; // r4
  int v17; // r0
  _DWORD *j; // r2
  unsigned __int8 v19; // r8
  _DWORD *k; // r3
  int v21; // r4
  _BYTE v22[36]; // [sp+0h] [bp-24h] BYREF

  result = (unsigned __int8)*a2;
  v7 = result - 5;
  if ( result - 5 <= 0xF )
  {
    result = *((unsigned __int16 *)&jpt_18734 + v7);
    switch ( v7 )
    {
      case 0u:
      case 2u:
        result = sub_16C9C((unsigned __int16 *)(a3 + 136), 0);
        if ( result == 0 )
        {
          *a2 = 2;
          result = 2;
        }
        break;
      case 1u:
        v8 = *(_DWORD **)sub_18AE8(a1 + 508, v22);
        v9 = 0;
        while ( v8 != *(_DWORD **)sub_18B00(a1 + 508, v22) && *(unsigned __int8 *)(v8[5] + 50) != *(_DWORD *)(a3 + 8) )
          v8 = sub_16A30(v8);
        if ( v8 == *(_DWORD **)sub_18B00(a1 + 508, v22) )
          v9 = 1;
        result = v9;
        if ( v9 != 0 )
          *a2 = 1;
        break;
      case 3u:
        v10 = *(_DWORD **)sub_18AE8(a1 + 508, v22);
        v11 = 1;
        while ( v10 != *(_DWORD **)sub_18B00(a1 + 508, v22) && *(_BYTE *)(v10[5] + 50) != 0 )
          v10 = sub_16A30(v10);
        if ( v10 != *(_DWORD **)sub_18B00(a1 + 508, v22) )
          v11 = 0;
        result = v11;
        if ( v11 != 0 )
        {
          *a2 = 4;
          result = 4;
        }
        break;
      case 4u:
      case 8u:
      case 9u:
      case 0xAu:
      case 0xDu:
      case 0xEu:
        return result;
      case 5u:
        v12 = *(_DWORD **)sub_18AE8(a1 + 508, v22);
        v13 = 1;
        while ( v12 != *(_DWORD **)sub_18B00(a1 + 508, v22) )
        {
          v14 = v12[5];
          if ( *(unsigned __int8 *)(v14 + 50) == *(_DWORD *)(a3 + 8) && *(unsigned __int8 *)(v14 + 49) > 1u )
            break;
          v12 = sub_16A30(v12);
        }
        if ( v12 != *(_DWORD **)(a1 + 508) )
          v13 = 0;
        result = v13;
        if ( v13 != 0 )
        {
          *a2 = 15;
          result = 15;
        }
        break;
      case 6u:
        v15 = 1;
        for ( i = *(_DWORD **)(*(_DWORD *)(a1 + 508) + 8); i != *(_DWORD **)(a1 + 508); i = sub_16A30(i) )
        {
          v17 = i[5];
          if ( *(unsigned __int8 *)(v17 + 50) == *(_DWORD *)(a3 + 8) )
          {
            for ( j = *(_DWORD **)(v17 + 28); j != *(_DWORD **)(v17 + 32) && *j == 0; ++j )
              ;
            if ( j != *(_DWORD **)(i[5] + 32) )
              break;
          }
        }
        if ( i != *(_DWORD **)(a1 + 508) )
          v15 = 0;
        result = v15;
        if ( v15 != 0 )
        {
          *a2 = 2;
          result = 2;
        }
        break;
      case 7u:
        v19 = 1;
        for ( k = *(_DWORD **)(*(_DWORD *)(a1 + 508) + 8);
              k != *(_DWORD **)(a1 + 508) && *(unsigned __int8 *)(k[5] + 50) != *(_DWORD *)(a3 + 8);
              k = sub_16A30(k) )
        {
          ;
        }
        if ( k != *(_DWORD **)(a1 + 508) )
          v19 = 0;
        result = v19;
        if ( v19 != 0 )
        {
          *a2 = 2;
          result = 2;
        }
        break;
      case 0xBu:
        result = sub_16C9C((unsigned __int16 *)(a3 + 136), 4);
        if ( result >= 0xA )
        {
          *a2 = 21;
          result = 21;
        }
        break;
      case 0xCu:
        result = sub_16C9C((unsigned __int16 *)(a3 + 136), 4);
        if ( result == 0 )
        {
          *a2 = 2;
          result = 2;
        }
        break;
      case 0xFu:
        v21 = sub_16C9C((unsigned __int16 *)(a3 + 136), 0);
        result = sub_16C9C((unsigned __int16 *)(a3 + 136), 3);
        if ( v21 == result )
        {
          *a2 = 1;
          result = 1;
        }
        break;
      default:
        abort();
    }
  }
  return result;
}

