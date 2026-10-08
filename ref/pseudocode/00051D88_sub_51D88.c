/* ea:   0x00051D88 */
/* name: sub_51D88 */
/* size: 412 */
unsigned int __fastcall sub_51D88(int a1)
{
  int v2; // r3
  unsigned int result; // r0
  _DWORD v4[3]; // [sp+0h] [bp-Ch] BYREF

  v2 = *(_DWORD *)(a1 + 12) + 20;
  result = *(unsigned __int8 *)(*(_DWORD *)(a1 + 12) + 22) << 26;
  if ( result >> 28 == 6 )
  {
    v4[0] = *(_BYTE *)(v2 + 3) & 0xF;
    if ( (unsigned __int8)sub_52998(v4, 0) != 0
      || (*(_BYTE *)(sub_53808(*(_DWORD *)(a1 + 24) + 52, *(_DWORD *)(a1 + 64), *(_DWORD *)(a1 + 68) - 1) + 22) & 3) != 1 )
    {
      if ( (unsigned __int8)sub_52998(v4, 1) != 0
        || (*(_BYTE *)(sub_53808(*(_DWORD *)(a1 + 24) + 52, *(_DWORD *)(a1 + 64) + 1, *(_DWORD *)(a1 + 68)) + 22) & 3) != 1 )
      {
        if ( (unsigned __int8)sub_52998(v4, 2) != 0
          || (*(_BYTE *)(sub_53808(*(_DWORD *)(a1 + 24) + 52, *(_DWORD *)(a1 + 64), *(_DWORD *)(a1 + 68) + 1) + 22) & 3) != 1 )
        {
          result = sub_52998(v4, 3);
          if ( (_BYTE)result == 0 )
          {
            result = sub_53808(*(_DWORD *)(a1 + 24) + 52, *(_DWORD *)(a1 + 64) - 1, *(_DWORD *)(a1 + 68));
            if ( (*(_BYTE *)(result + 22) & 3) == 1 )
            {
              *(_BYTE *)(a1 + 129) = 8;
              *(_BYTE *)(a1 + 130) = 8;
              *(_DWORD *)(a1 + 124) = 0;
              return 0;
            }
          }
        }
        else
        {
          *(_BYTE *)(a1 + 129) = 4;
          *(_BYTE *)(a1 + 130) = 4;
          *(_DWORD *)(a1 + 124) = 1;
          return 1;
        }
      }
      else
      {
        *(_BYTE *)(a1 + 129) = 2;
        *(_BYTE *)(a1 + 130) = 2;
        *(_DWORD *)(a1 + 124) = 3;
        return 3;
      }
    }
    else
    {
      *(_BYTE *)(a1 + 129) = 1;
      *(_BYTE *)(a1 + 130) = 1;
      *(_DWORD *)(a1 + 124) = 2;
      return 2;
    }
  }
  return result;
}

