/* ea:   0x00064EB8 */
/* name: sub_64EB8 */
/* size: 56 */
_BYTE *sub_64EB8()
{
  _BYTE *v0; // r3
  _BYTE *result; // r0

  v0 = operator new(0x1Cu);
  result = nullptr;
  if ( v0 != nullptr )
  {
    v0[12] = 0;
    *((_DWORD *)v0 + 4) = 0;
    *((_DWORD *)v0 + 6) = 0;
    *(_DWORD *)v0 = &off_7BCF0;
    return v0;
  }
  return result;
}

