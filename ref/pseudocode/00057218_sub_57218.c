/* ea:   0x00057218 */
/* name: sub_57218 */
/* size: 92 */
int __fastcall sub_57218(int result)
{
  bool v1; // zf

  if ( (unsigned int)(*(unsigned __int8 *)(*(_DWORD *)(result + 12) + 22) << 26) >> 28 != 1
    || (v1 = ((unsigned int)(*(unsigned __int16 *)(*(_DWORD *)(result + 12) + 24) << 24) >> 30) + 1 == *(_DWORD *)(result + 216),
        LOBYTE(result) = 1,
        v1) )
  {
    LOBYTE(result) = 0;
  }
  return (unsigned __int8)result;
}

